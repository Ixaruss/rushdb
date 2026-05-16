use crate::{conf::Config, engine::Store};
use bincode::{self};
use std::{collections::HashMap, fs};

pub fn save(store: &HashMap<String, String>, conf: Config) -> std::io::Result<()> {
    let bytes = bincode::serialize(store).expect("Failed to serialize");
    fs::create_dir_all(conf.server.save_dir)?;

    match fs::write(conf.server.save_path, bytes) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn load(conf: Config) -> Store {
    match fs::read(conf.server.save_path) {
        Ok(bytes) => bincode::deserialize(&bytes).expect("Failed to deserialize"),
        Err(_) => Store::new(),
    }
}
