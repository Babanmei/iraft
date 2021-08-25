use std::collections::HashMap;

pub struct Config {
    pub id: String,
    pub peers: HashMap<String, String>,
    pub listen_raft: String,
    pub log_level: String,
    pub data_dir: String,
}

impl Config {
    pub fn new() -> Config {
        let mut peer = HashMap::new();
        let x = "1";
        let y = "2";
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
