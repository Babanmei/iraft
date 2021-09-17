mod transport;
mod store;
mod log;
pub mod server;
pub mod memory_store;
pub mod conf;
pub mod message;
pub mod node;

#[cfg(feature = "yaml")]

pub trait Raft {

}

pub trait Store: std::fmt::Debug + Sync + Send {

}