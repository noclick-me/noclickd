use std::collections::HashMap;
use std::sync::RwLock;

use crate::config::Config;

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub source_url: String,
    pub noclick_url: String,
}

pub struct SharedState {
    pub db: RwLock<HashMap<String, Entry>>,
    pub config: Config,
}

impl SharedState {
    pub fn new(config: Config) -> Self {
        SharedState {
            db: RwLock::new(HashMap::new()),
            config,
        }
    }
}
