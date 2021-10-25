use std::collections::HashMap;
use anyhow::Result;
use std::fs::File;
use std::io::Read;
use serde_derive::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Config {
    pub id: String,
    pub peers: HashMap<String, String>,
    pub listen_raft: String,
    pub log_level: String,
    pub data_dir: String,
}

impl Config {

    pub fn new(file: &str) -> Result<Config> {
        let mut s = String::new();
        let file = std::env::current_dir()?.join(file);
        let mut f = File::open(file)?;
        let _ = f.read_to_string(&mut s)?;

        let conf: Config = serde_yaml::from_str(&s)?;
        Ok(conf)
    }

    pub fn default() -> Config {
        let mut peer = HashMap::new();
        let x = "2";
        let y = "1";
        peer.insert(x.clone().to_string(), "127.0.0.1:111".to_owned() + x);
        Config {
            id: y.clone().to_string(),
            peers: peer,
            listen_raft: "127.0.0.1:111".to_owned() + y,
            log_level: "debug".to_string(),
            data_dir: "/data/iraft".to_owned(),
        }
    }
}
