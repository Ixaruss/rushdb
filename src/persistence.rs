use crate::{conf::Config, engine::Store};
use bincode::{self};
use std::{collections::HashMap, fs};

const SAVE_PATH: &str = "./data/snapshot.bin";
const SAVE_DIR: &str = "./data";

pub fn save(store: &HashMap<String, String>, conf: Option<Config>) -> std::io::Result<()> {
    let bytes = bincode::serialize(store).expect("Failed to serialize");
    let dir = if conf.clone().unwrap().server.save_dir.is_empty() {
        SAVE_DIR.to_string()
    } else {
        conf.clone().unwrap().server.save_dir
    };
    fs::create_dir_all(dir)?;

    let path = if conf.clone().unwrap().server.save_path.is_empty() {
        SAVE_PATH.to_string()
    } else {
        conf.unwrap().server.save_path
    };
    match fs::write(path, bytes) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn load(conf: Option<Config>) -> Store {
    let path = if conf.clone().unwrap().server.save_path.is_empty() {
        SAVE_PATH.to_string()
    } else {
        conf.unwrap().server.save_path
    };
    match fs::read(path) {
        Ok(bytes) => bincode::deserialize(&bytes).expect("Failed to deserialize"),
        Err(_) => Store::new(),
    }
}
