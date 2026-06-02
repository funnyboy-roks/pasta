use std::collections::HashSet;

use anyhow::Context;
use async_compression::tokio::bufread::GzipDecoder;
use futures_util::StreamExt;
use sqlx::{SqliteExecutor, prelude::FromRow};
use time::OffsetDateTime;

#[derive(Debug, sqlx::FromRow)]
pub struct NewPaste<'a> {
    pub slug: &'a Slug,
    pub path: &'a str,
    pub hash: Hash,
    pub content_type: Option<&'a str>,
    pub expires_in: &'a str,
}

impl NewPaste<'_> {
    pub async fn insert(self, db: impl SqliteExecutor<'_>) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO pastes (slug, path, hash, content_type, expires_at)
            VALUES (?, ?, ?, ?, datetime('now', '+' || ?))
            "#,
            self.slug,
            self.path,
            self.hash,
            self.content_type,
            self.expires_in,
        )
        .execute(db)
        .await
        .context("Inserting paste")?;
        Ok(())
    }
}

#[derive(Debug, FromRow)]
pub struct PasteWithUniq {
    #[sqlx(flatten)]
    pub paste: Paste,
    pub uniq: bool,
}

#[derive(Debug, FromRow)]
#[allow(unused)]
pub struct Paste {
    pub slug: Slug,
    pub path: String,
    pub hash: Hash,
    pub content_type: Option<String>,
    pub created_at: OffsetDateTime,
    pub expires_at: Option<OffsetDateTime>,
}

/// Database operation implementations
impl Paste {
    pub async fn get_one(db: impl SqliteExecutor<'_>, slug: &Slug) -> anyhow::Result<Option<Self>> {
        sqlx::query_as! {
            Self,
            "SELECT * FROM pastes WHERE slug = ?",
            slug
        }
        .fetch_optional(db)
        .await
        .context("Getting paste by slug")
    }

    pub async fn get_by_hash(
        db: impl SqliteExecutor<'_>,
        hash: &Hash,
    ) -> anyhow::Result<Vec<Self>> {
        sqlx::query_as!(
            Paste,
            r#"
            SELECT * FROM pastes
            WHERE hash = ?
            "#,
            hash
        )
        .fetch_all(db)
        .await
        .context("Getting matching hashes")
    }

    pub async fn get_expired(db: impl SqliteExecutor<'_>) -> anyhow::Result<Vec<PasteWithUniq>> {
        sqlx::query_as(
            r#"
            SELECT *, hash in (SELECT hash FROM pastes GROUP BY hash HAVING COUNT(hash) = 1) uniq
            FROM pastes
            WHERE expires_at < CURRENT_TIMESTAMP;
            "#,
        )
        .fetch_all(db)
        .await
        .context("Getting expied pastes")
    }

    pub async fn delete(&self, db: impl SqliteExecutor<'_>) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM pastes WHERE slug = ?
            "#,
            self.slug,
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn get_known_paths(db: impl SqliteExecutor<'_>) -> anyhow::Result<HashSet<String>> {
        let mut stream = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT path FROM pastes;
            "#,
        )
        .fetch(db);

        let mut out = HashSet::with_capacity(stream.size_hint().0);
        while let Some(q) = stream.next().await {
            let q = q?;
            out.insert(q);
        }
        Ok(out)
    }
}

impl Paste {
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .is_some_and(|e| e < OffsetDateTime::now_utc())
    }

    pub async fn get_content(
        &self,
        state: &AppState,
    ) -> anyhow::Result<(u64, impl AsyncBufRead + 'static)> {
        let mut file = tokio::fs::File::open(state.data_dir.join(&self.path))
            .await
            .context("Opening file")?;
        let len = file
            .seek(std::io::SeekFrom::End(0))
            .await
            .context("getting file length")?;
        file.seek(std::io::SeekFrom::Start(0))
            .await
            .context("getting file length")?;
        let file = BufReader::new(file);
        Ok((len, file))
    }

    pub async fn get_content_decoded(&self, state: &AppState) -> anyhow::Result<String> {
        let (content_len, file) = self.get_content(state).await?;
        let mut s = String::with_capacity(content_len as _);
        let mut dec = GzipDecoder::new(file);
        dec.read_to_string(&mut s)
            .await
            .context("reading encrypted content")?;
        Ok(s)
    }
}

pub use slug::Slug;
mod slug {
    use std::fmt::Debug;

    use anyhow::{Context, bail};
    use axum::response::IntoResponse;
    use rand::RngExt;
    use serde::Deserialize;
    use sqlx::{Database, Decode, Encode, Sqlite, SqliteExecutor, Type};

    use crate::util::LowerAlphanumeric;

    #[derive(Clone, Copy)]
    pub struct Slug {
        value: [u8; 32],
        len: usize,
    }

    impl Debug for Slug {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("Slug").field(&self.as_str()).finish()
        }
    }

    impl Slug {
        /// # SAFETY
        ///
        /// value[..len] must be initialised with valid utf-8 characters
        unsafe fn from_raw_parts(value: [u8; 32], len: usize) -> Self {
            Slug { value, len }
        }

        pub async fn new_unique(
            db: impl SqliteExecutor<'_> + Copy,
        ) -> anyhow::Result<Option<Self>> {
            for limit in (8..=32).step_by(2) {
                let slug = Self::new(limit);

                if !slug.exists(db).await? {
                    return Ok(Some(slug));
                }
            }

            Ok(None)
        }

        fn new(len: usize) -> Self {
            debug_assert!(len <= 32);
            let mut slug = [0u8; 32];
            let mut rng = rand::rng();
            slug[..len].fill_with(|| rng.sample(LowerAlphanumeric) as u8);
            // SAFETY: slug is filled up to len by LowerAlphanumeric, which only yields valid ascii
            // chars
            unsafe { Self::from_raw_parts(slug, len) }
        }

        pub async fn exists(&self, db: impl SqliteExecutor<'_>) -> anyhow::Result<bool> {
            let existing = sqlx::query_scalar!(
                r#"
                select COUNT(*) FROM pastes WHERE slug = ? LIMIT 1
                "#,
                self
            )
            .fetch_one(db)
            .await
            .context("Getting slug")?;
            Ok(existing != 0)
        }

        pub fn parse(s: &str) -> anyhow::Result<Self> {
            if !(3..=32).contains(&s.len()) {
                bail!("Slug length must be within 3..=32");
            }

            if s.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
            {
                assert!(s.len() < 32);
                let mut slug = [0u8; 32];
                slug[..s.len()].copy_from_slice(&s.as_bytes()[..s.len()]);
                // SAFETY: String can only contain valid utf-8
                Ok(unsafe { Self::from_raw_parts(slug, s.len()) })
            } else {
                bail!("Invalid character in slug");
            }
        }

        /// Generate a path for this slug (including random suffix)
        // NOTE: this returns a string since sqlx doesn't support serialising paths
        pub fn to_path(self) -> String {
            const RAND_LEN: usize = 5;
            let mut s = String::with_capacity(self.len + '-'.len_utf8() + RAND_LEN);
            s.push_str(self.as_str());
            s.push('-');
            rand::rng()
                .sample_iter(LowerAlphanumeric)
                .take(5)
                .for_each(|c| s.push(c));
            s
        }

        pub fn as_str(&self) -> &str {
            // SAFETY: The only way to initialise this structure is to use methods that ensure utf-8
            unsafe { str::from_utf8_unchecked(&self.value[..self.len]) }
        }
    }

    impl<'de> Deserialize<'de> for Slug {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = <&str>::deserialize(deserializer)?;
            Self::parse(s).map_err(serde::de::Error::custom)
        }
    }

    impl From<String> for Slug {
        fn from(value: String) -> Self {
            Self::parse(&value).expect("Invalid string parsed as Slug")
        }
    }

    impl Type<Sqlite> for Slug {
        fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
            <&str as Type<Sqlite>>::type_info()
        }
    }

    impl<'r> Decode<'r, Sqlite> for Slug {
        fn decode(
            value: <Sqlite as Database>::ValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            let s = <&str as Decode<'r, Sqlite>>::decode(value)?;
            Ok(Self::parse(s).map_err(|e| Box::new(std::io::Error::other(e)))?)
        }
    }

    impl<'q> Encode<'q, Sqlite> for Slug {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as Database>::ArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            <&str as Encode<'q, Sqlite>>::encode_by_ref(&self.as_str(), buf)
        }
    }

    impl IntoResponse for Slug {
        fn into_response(self) -> axum::response::Response {
            self.as_str().to_string().into_response()
        }
    }
}

pub use hash::Hash;
use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncSeekExt, BufReader};

use crate::AppState;
mod hash {
    use anyhow::anyhow;
    use axum::http::HeaderValue;
    use sqlx::{Database, Decode, Encode, Sqlite, Type};

    #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
    pub struct Hash([u8; 32]);

    impl Hash {
        pub fn new(raw: impl Into<[u8; 32]>) -> Self {
            Self(raw.into())
        }

        pub fn as_base64(&self) -> String {
            use base64::prelude::*;
            BASE64_STANDARD.encode(self.0)
        }

        pub fn to_header(self) -> HeaderValue {
            HeaderValue::from_str(&format!("sha256=:{}:", self.as_base64()))
                .expect("all characters in base64 and in string are be valid")
        }
    }

    impl From<Vec<u8>> for Hash {
        fn from(value: Vec<u8>) -> Self {
            assert_eq!(value.len(), 32, "Invalid length for hash");

            let mut hash = [0u8; 32];
            hash.copy_from_slice(&value);
            Self::new(hash)
        }
    }

    impl<'r> Decode<'r, Sqlite> for Hash {
        fn decode(
            value: <Sqlite as Database>::ValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            let s = <&[u8] as Decode<'r, Sqlite>>::decode(value)?;
            if s.len() != 32 {
                return Err(anyhow!("Invalid length for hash").into());
            }

            let mut hash = [0u8; 32];
            hash.copy_from_slice(s);

            Ok(Self::new(hash))
        }
    }

    impl<'q> Encode<'q, Sqlite> for Hash {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as Database>::ArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            <&[u8] as Encode<'q, Sqlite>>::encode_by_ref(&&self.0[..], buf)
        }
    }

    impl Type<Sqlite> for Hash {
        fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
            <[u8] as Type<Sqlite>>::type_info()
        }
    }
}
