use anyhow::Result;

pub struct Config {
    pub database_url: String,
    pub google_application_credentials: String,
}

pub fn load_config() -> Result<Config> {
    let database_url = dotenv::var("DATABASE_URL")?;
    let google_application_credentials = dotenv::var("GOOGLE_APPLICATION_CREDENTIALS")?;
    Ok(Config {
        database_url,
        google_application_credentials,
    })
}
