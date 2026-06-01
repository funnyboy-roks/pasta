use std::path::Path;

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("cargo::rerun-if-changed=migration.sql");
    let cargo_target_dir =
        std::env::var("OUT_DIR").context("Failed to get cargo target directory")?;

    let path = Path::new(&cargo_target_dir).join("initial_data.db");

    let sqlite_uri = format!("sqlite:{}", path.to_str().unwrap());

    println!("cargo::rustc-env=DATABASE_URL={}", sqlite_uri);

    tokio::fs::File::create(path)
        .await
        .context("Failed to create db")?;

    let db = sqlx::sqlite::SqlitePool::connect(&sqlite_uri)
        .await
        .context("Failed to create database layer")?;

    sqlx::migrate!("./migrations").run(&db).await?;

    Ok(())
}
