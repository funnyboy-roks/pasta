use std::{sync::Arc, time::Duration};

use anyhow::Context;
use async_walkdir::WalkDir;
use futures_util::StreamExt;
use tokio::time::Instant;
use tracing::{Instrument, debug, error, info, info_span};

use crate::{AppState, Paste};

/// Schedule the cleanup tasks with the tokio runtime
pub fn schedule(state: Arc<AppState>) {
    // every 10 minutes, remove expired pastes
    tokio::spawn({
        let state = Arc::clone(&state);
        async move {
            let mut interval = tokio::time::interval(Duration::from_mins(10));
            loop {
                interval.tick().await;
                if let Err(e) = cleanup(Arc::clone(&state)).await {
                    error!("{:?}", e)
                }
            }
        }
    });

    // every 2 hours, walk the data dir and remove any orphaned files
    // (these shouldn't happen, but I would like to be certain)
    tokio::spawn({
        let state = Arc::clone(&state);
        async move {
            let mut interval = tokio::time::interval_at(
                // offset slightly so they two cleanups never happen at the same time
                Instant::now() + Duration::from_mins(1),
                Duration::from_hours(2),
            );
            loop {
                interval.tick().await;
                if let Err(e) = cleanup_orphans(Arc::clone(&state)).await {
                    error!("{:?}", e)
                }
            }
        }
    });
}

#[tracing::instrument(skip(state))]
async fn cleanup(state: Arc<AppState>) -> anyhow::Result<()> {
    let expired = Paste::get_expired(&state.pool).await?;
    info!(expired = expired.len(), "starting cleanup");

    for e in expired {
        let state = state.clone();
        let slug = e.paste.slug;
        let result = async move {
            if e.uniq {
                info!("deleting file as paste is unique");
                let path = state.data_dir.join(&e.paste.path);
                tokio::fs::remove_file(&path)
                    .await
                    .with_context(|| format!("deleting file '{}'", path.display()))?;
            }

            debug!("Deleting from database");
            e.paste
                .delete(&state.pool)
                .await
                .context("Deleting expired paste")?;

            anyhow::Ok(())
        }
        .instrument(info_span!("cleaning up", ?slug))
        .await;

        match result {
            Ok(()) => {}
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip(state))]
async fn cleanup_orphans(state: Arc<AppState>) -> anyhow::Result<()> {
    info!("starting orphan cleanup");

    let paths = Paste::get_known_paths(&state.pool).await?;

    let mut walk = WalkDir::new(&state.data_dir);

    let mut removed = 0;
    let mut errored = 0;
    while let Some(w) = walk.next().await {
        let w = w?;
        if w.metadata().await?.is_dir() {
            continue;
        }
        let path = w.path();
        let trimmed_path = path.strip_prefix(&state.data_dir)?;

        let Some(trimmed_path) = trimmed_path.to_str() else {
            continue;
        };

        if paths.contains(trimmed_path) {
            // this path is used by a paste
            continue;
        }

        match tokio::fs::remove_file(&path).await {
            Ok(()) => {
                removed += 1;
            }
            Err(error) => {
                errored += 1;
                error!(path=%path.display(), ?error, "Unable to remove file");
            }
        }
    }

    info!(removed, errored, "completed orphan cleanup");

    Ok(())
}
