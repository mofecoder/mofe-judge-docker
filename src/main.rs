#![allow(renamed_and_removed_lints)]
#![allow(clippy::unknown_clippy_lints)]

mod api;
mod checker;
mod command;
mod config;
mod db;
mod gcp;
mod models;
mod sandbox;

#[macro_use]
extern crate rocket;

use anyhow::Result;
use api::{
    judge::judge,
    download::download,
    compile::compile,
};
use config::Config;
use once_cell::sync::Lazy;
use std::sync::Arc;

static CONFIG: Lazy<Config> = Lazy::new(|| config::load_config().unwrap());

const MAX_FILE_SIZE: usize = 200_000_000; // 200MB
#[allow(dead_code)]
const MAX_MEMORY_USAGE: i32 = 1_024_000; // 1024MB

#[rocket::main]
async fn main() -> Result<()> {
    let conn = {
        let pool = db::new_pool(&CONFIG).await?;
        Arc::new(pool)
    };

    rocket::ignite()
        .manage(conn)
        .mount("/", routes![judge, download, compile])
        .launch()
        .await?;

    Ok(())
}
