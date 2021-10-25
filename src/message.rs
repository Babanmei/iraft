use serde_derive::{Deserialize, Serialize};

/// A message address.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Address {
    /// Broadcast to all peers.
    Peers,
    /// A remote peer.
    Peer(String),
    /// The local node.
    Local,
    /// A local client.
    Client,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Heartbeat {
        commit_index: u64,
        commit_term: u64,
    },
    //candidate 请求投票
    SolicitVote {
        last_index: u64,
        last_term: u64,
    },
    //只要你敢拉票, 我就敢支持
    GrantVote,
    //对leader心跳的回应
    ConfirmLeader {
        commit_index: u64,
        has_committed: bool,
    },
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub term: u64,
    pub from: Address,
    pub to: Address,
    pub event: Event,
}


//pub type Message = Vec<u8>;
