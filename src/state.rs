use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub source_url: String,
    pub noclick_url: String,
}

pub struct SharedState {
    pub db: RwLock<HashMap<String, Entry>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            db: RwLock::new(HashMap::new()),
        }
    }
}
