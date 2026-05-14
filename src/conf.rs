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
        let default_path: String = String::from("/etc/bcc/conf.toml");
        let p = match path {
            Some(val) => val,
            None => default_path,
        };
        let res = toml::from_str(match &fs::read_to_string(p) {
            Ok(val) => val,
            Err(e) => {
                println!("cant parse conf file: {}", e);
                ""
            }
        });
        let config: Config = match res {
            Ok(c) => c,
            Err(e) => {
                println!("error while reading: {}", e);
                Config {
                    server: ServerConfig {
                        host: String::new(),
                        port: 0,
                        save_interval: 0,
                        save_path: String::new(),
                        save_dir: String::new(),
                    },
                }
            }
        };
        println!("loaded config: {:?}", config);
        config
    }
}
