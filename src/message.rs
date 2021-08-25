
use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Heartbeat {
        commit_index: u64,
        commit_term: u64,
    },
    None
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Message {
    pub term: u64,
    pub event: Event,
}


//pub type Message = Vec<u8>;
