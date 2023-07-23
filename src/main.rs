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

use crate::gcp::GcpClient;
use config::Config;
use google_cloud_storage::client::{Client, ClientConfig};
use once_cell::sync::Lazy;
use std::sync::Arc;

static CONFIG: Lazy<Config> = Lazy::new(|| config::load_config().unwrap());

const MAX_FILE_SIZE: usize = 200_000_000; // 200MB
#[allow(dead_code)]
const MAX_MEMORY_USAGE: i32 = 1_024_000; // 1024MB
const MAX_STDERR_SIZE: usize = 5_000; // 5KB

static JUDGE_DIR: Lazy<std::path::PathBuf> =
    Lazy::new(|| std::env::current_dir().unwrap().join("judge"));

#[launch]
async fn rocket() -> _ {
    let conn = {
        let pool = db::new_pool(&*CONFIG).await.unwrap();
        Arc::new(pool)
    };

    let client = Client::new(ClientConfig::default().with_auth().await.unwrap());

    let gcp_client = GcpClient::new(client).await.unwrap();

    rocket::build()
        .manage(conn)
        .manage(Arc::new(gcp_client))
        .mount(
            "/",
            routes![
                api::judge::judge,
                api::download::download,
                api::compile::compile
            ],
        )
}
