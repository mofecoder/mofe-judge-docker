mod api;
mod model;
mod command;
mod gcp;

#[macro_use]
extern crate rocket;

use anyhow::Result;

#[rocket::main]
async fn main() -> Result<()> {
    rocket::ignite().mount("/", routes![]).launch().await?;

    Ok(())
}
