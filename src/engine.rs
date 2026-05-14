use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Store {
    pub db: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        let ins: HashMap<String, String> = HashMap::new();
        return Store { db: ins };
    }
}
