#![allow(clippy::unknown_clippy_lints)]

mod api;
mod command;
mod config;
mod db;
mod gcp;
mod models;
mod sandbox;

#[macro_use]
extern crate rocket;

use anyhow::Result;
use api::judge::judge;
use config::Config;
use once_cell::sync::Lazy;

static CONFIG: Lazy<Config> = Lazy::new(|| config::load_config().unwrap());

const MAX_FILE_SIZE: usize = 200_000_000; // 200MB
const MAX_MEMORY_USAGE: i32 = 1_024_000; // 1024MB

#[rocket::main]
async fn main() -> Result<()> {
    rocket::ignite().mount("/", routes![judge]).launch().await?;

    Ok(())
}
