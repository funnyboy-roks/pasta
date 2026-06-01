use std::{fmt::Debug, net::SocketAddr, ops::Not, path::PathBuf, pin::pin, sync::Arc};

use anyhow::Context;
use async_compression::{
    Level,
    tokio::bufread::{GzipDecoder, GzipEncoder},
};
use axum::{
    Router,
    body::{Body, BodyDataStream},
    extract::{Path, Request, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE},
    },
    routing::{get, get_service, post},
};
use futures_util::{Stream, TryStreamExt};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
};
use tokio_util::io::{ReaderStream, StreamReader};
use tower_http::{limit::RequestBodyLimitLayer, services::ServeDir, trace::TraceLayer};
use tracing::{info, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    db::{Hash, NewPaste, Paste, Slug},
    error::HttpError,
};

mod cleanup;
mod db;
mod error;
mod util;

/// 10 KB
const MAX_CONTENT_LENGTH: usize = 10 * 1000;

struct AppState {
    pool: SqlitePool,
    data_dir: PathBuf,
    db_file: PathBuf,
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("pool", &"..")
            .field("data_dir", &self.data_dir)
            .finish()
    }
}

#[tracing::instrument(skip(headers, state))]
#[axum::debug_handler]
async fn get_one(
    Path(slug): Path<Slug>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Body), HttpError> {
    let Some(paste) = Paste::get_one(&state.pool, &slug).await? else {
        return Err(StatusCode::NOT_FOUND.into());
    };

    if paste.is_expired() {
        return Err(StatusCode::NOT_FOUND.into());
    }

    let accept_gzip = if let Some(accept_encoding) = headers.get(ACCEPT_ENCODING)
        && let Ok(accept_encoding) = accept_encoding.to_str()
    {
        accept_encoding.contains("gzip")
    } else {
        false
    };

    let file = tokio::fs::File::open(state.data_dir.join(paste.path))
        .await
        .context("Opening file")?;
    let file = BufReader::new(file);

    let mut headers = HeaderMap::new();
    headers.insert(
        "content-digest",
        HeaderValue::from_str(&format!("sha256=:{}:", paste.hash.as_base64()))
            .expect("all characters in base64 and in string are be valid"),
    );
    if let Some(c) = paste.content_type {
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&c).context("Parsing content_type")?,
        );
    }

    let body = if accept_gzip {
        headers.insert(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        let stream = ReaderStream::new(file);
        Body::from_stream(stream)
    } else {
        let dec = GzipDecoder::new(file);
        let stream = ReaderStream::new(dec);
        Body::from_stream(stream)
    };
    Ok((headers, body))
}

#[tracing::instrument]
async fn get_matching(
    state: &AppState,
    hash: &Hash,
    content: &[u8],
) -> anyhow::Result<Option<Paste>> {
    let existing = Paste::get_by_hash(&state.pool, hash).await?;

    'outer: for e in existing {
        let path = state.data_dir.join(&e.path);
        if !tokio::fs::try_exists(&path)
            .await
            .context("Checking if path exists")?
        {
            continue;
        }

        debug_assert_eq!(e.hash, *hash);

        let len = tokio::fs::metadata(&path)
            .await
            .context("Getting file metadata")?
            .len();

        if len != content.len() as u64 {
            continue;
        }

        let file = tokio::fs::File::open(path)
            .await
            .context("Opening matching file")?;
        let mut file = BufReader::new(file);

        let mut rest = content;

        loop {
            let file_buf = file.fill_buf().await.context("reading matching file")?;

            if file_buf.is_empty() {
                if rest.is_empty() {
                    return Ok(Some(e));
                } else {
                    continue 'outer;
                }
            }

            let len = file_buf.len();

            if &rest[..len] == file_buf {
                file.consume(len);
                rest = &rest[len..];
            } else {
                continue 'outer;
            }
        }
    }

    Ok(None)
}

#[tracing::instrument(skip(headers, state, body))]
async fn create_paste(
    slug: &Slug,
    headers: HeaderMap,
    state: Arc<AppState>,
    body: BodyDataStream,
) -> Result<(), HttpError> {
    let mut compressed = Vec::with_capacity(body.size_hint().0);

    let mut body = pin!(StreamReader::new(body.map_err(std::io::Error::other)));
    let mut hash = Sha256::new();

    if let Some(e) = headers.get(CONTENT_ENCODING) {
        if e == "gzip" {
            trace!("already gzipped");

            body.take(MAX_CONTENT_LENGTH as _)
                .read_to_end(&mut compressed)
                .await
                .context("Reading body")?;

            hash.update(&compressed);
        } else {
            trace!(encoding=?e, "unknown encoding");
            return Err(HttpError::from(StatusCode::BAD_REQUEST));
        }
    } else {
        trace!("no content encoding specified");
        let mut encoder = GzipEncoder::with_quality(&mut body, Level::Best);

        let mut nread = 0;
        let mut buf = [0u8; 1024];
        loop {
            let n = encoder.read(&mut buf).await.context("reading body")?;
            if n == 0 {
                break;
            }

            // if the body is longer than the content-length header, bail
            nread += n;
            if nread > MAX_CONTENT_LENGTH {
                return Err(HttpError::from(StatusCode::PAYLOAD_TOO_LARGE));
            }

            let read = &buf[..n];
            hash.update(read);
            compressed.extend(read);
        }
    };

    let hash: Hash = Hash::new(hash.finalize());

    let existing = get_matching(&state, &hash, &compressed).await?;

    let path = if let Some(existing) = existing {
        existing.path
    } else {
        let path = slug.to_path();

        let full_path = state.data_dir.join(&path);

        tokio::fs::create_dir_all(full_path.parent().expect("always in data_dir"))
            .await
            .context("Creating directory for file")?;

        tokio::fs::write(&full_path, &compressed)
            .await
            .context("Creating file")?;

        path
    };

    // TODO: inserting after checking for slug could cause race, but it's fast enough that it's not critical

    let header = headers
        .get(CONTENT_TYPE)
        .map(|v| v.to_str())
        .transpose()
        .map_err(|e| HttpError::new(e, StatusCode::BAD_REQUEST))?;

    NewPaste {
        slug,
        path: &path,
        hash,
        content_type: header,
        expires_in: "7 days",
    }
    .insert(&state.pool)
    .await?;

    Ok(())
}

#[axum::debug_handler]
async fn new_paste(
    slug: Option<Path<Slug>>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<(StatusCode, Slug), HttpError> {
    let slug = if let Some(Path(slug)) = slug {
        slug.exists(&state.pool).await?.not().then_some(slug)
    } else {
        Slug::new_unique(&state.pool).await?
    };

    let Some(slug) = slug else {
        return Err(StatusCode::CONFLICT.into());
    };

    let Some(content_length) = headers.get(CONTENT_LENGTH).and_then(|l| l.to_str().ok()) else {
        return Err(StatusCode::LENGTH_REQUIRED.into());
    };

    let Ok(content_length) = content_length.parse::<usize>() else {
        return Err(StatusCode::LENGTH_REQUIRED.into());
    };

    if content_length > MAX_CONTENT_LENGTH {
        return Err(StatusCode::PAYLOAD_TOO_LARGE.into());
    }

    let body = request.into_body().into_data_stream();
    create_paste(&slug, headers, state, body).await?;

    Ok((StatusCode::CREATED, slug))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                concat!(env!("CARGO_CRATE_NAME"), "=info,tower_http=debug").into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_file: PathBuf = std::env::var("DB_FILE")
        .context("DB_FILE env var not set")?
        .into();

    // create database if it doesn't exist
    OpenOptions::new()
        .append(true)
        .create(true)
        .open(&db_file)
        .await
        .context("Creating database")?;

    let state = Arc::new(AppState {
        pool: SqlitePool::connect(&format!("sqlite:{}", db_file.display())).await?,
        data_dir: std::env::var("DATA_DIR")
            .context("DATA_DIR env var not set")?
            .into(),
        db_file,
    });

    sqlx::migrate!("./migrations").run(&state.pool).await?;

    cleanup::schedule(Arc::clone(&state));

    let web_ui_path = std::env::var("WEB_UI").ok().map(PathBuf::from);

    // build our application with a single route
    let mut app = Router::new()
        .route("/{slug}", get(get_one))
        .route("/", post(new_paste).put(new_paste))
        .route("/post", post(new_paste).put(new_paste))
        .route("/{slug}", post(new_paste).put(new_paste))
        .layer(RequestBodyLimitLayer::new(MAX_CONTENT_LENGTH))
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    if let Some(web_ui_path) = web_ui_path {
        let serve_dir = ServeDir::new(&web_ui_path);
        app = app
            .fallback_service(serve_dir.clone())
            .nest_service("/_app", ServeDir::new(web_ui_path.join("_app")))
            .route("/", get_service(serve_dir));
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Listening at {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}
