use std::fs;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
}
#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub save_interval: u64,
    pub save_path: String,
    pub save_dir: String,
}

impl Config {
    pub fn load(path: Option<String>) -> Self {
        let default_path: String = String::from("/etc/conf.toml");
        let p = match path {
            Some(val) => val,
            None => default_path,
        };
        let default_config = || Config {
            server: ServerConfig {
                host: String::from("0.0.0.0"),
                port: 6080,
                save_interval: 60,
                save_path: String::from("./data/snapshot.bin"),
                save_dir: String::from("./data"),
            },
        };

        let config: Config = match fs::read_to_string(p) {
            Ok(val) => toml::from_str(&val).unwrap_or_else(|_| default_config()),
            Err(_) => default_config(),
        };

        println!("loaded config: {:?}", config);
        config
    }
}
