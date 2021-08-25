mod node;
mod transport;
mod store;
mod log;
pub mod server;
pub mod memory_store;
pub mod config;
pub mod message;

pub trait Raft {

}

pub trait Store: std::fmt::Debug/*raftnode {:#?}*/{

}