use anyhow::Result;
use std::env;

pub struct Config {
    pub database_url: String,
    pub google_application_credentials: String,
}

pub fn load_config() -> Result<Config> {
    let database_url = env::var("DATABASE_URL")?;
    let google_application_credentials = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
    Ok(Config {
        database_url,
        google_application_credentials,
    })
}
