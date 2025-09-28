use sled::{Config, Db, Mode};

use crate::client::Client;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub db: Db,
}

impl AppState {
    pub fn new(client: Client, db: Db) -> Self {
        Self { client, db }
    }

    pub fn default() -> Self {
        let client = Client::new();
        let db = Config::default()
            .path("cache")
            .flush_every_ms(Some(1000))
            .mode(Mode::HighThroughput)
            .open()
            .unwrap();

        Self::new(client, db)
    }

    pub fn try_default() -> Result<Self, sled::Error> {
        let client = Client::new();
        let db = Config::default()
            .path("cache")
            .flush_every_ms(Some(1000))
            .mode(Mode::HighThroughput)
            .open()?;

        Ok(Self::new(client, db))
    }
}