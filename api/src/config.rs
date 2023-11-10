use std::env;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub database_url: String,

    pub max_simultaneous_downloads: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            database_url: "sqlite::memory:".to_string(),
            max_simultaneous_downloads: 10,
        }
    }
}

impl ServerConfig {
    pub fn load() -> Self {
        let mut config = Self::default();

        if let Ok(raw_port) = env::var("PORT") {
            config.port = raw_port.parse::<u16>().expect("PORT must be a number")
        }

        if let Ok(url) = env::var("DATABASE_URL") {
            config.database_url = url;
        }

        if let Ok(amounts) = env::var("MAX_SIMULTANEOUS_DOWNLOADS") {
            config.max_simultaneous_downloads = amounts
                .parse::<usize>()
                .expect("MAX_SIMULTANEOUS_DOWNLOADS must be a number")
        }

        config
    }
}
