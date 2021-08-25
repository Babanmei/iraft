use crate::log::Log;
use crate::message::{Message, Event};
use anyhow::Result;
use crate::Store;
use futures::channel::mpsc::{UnboundedSender};


#[derive(Debug)]
pub enum NodeRole {
    Candidate,
    Follower,
    Leader,
}

#[derive(Debug)]
pub struct RaftNode<S> where S: Store {
    id: String,
    peers: Vec<String>,
    term: u64,
    pub log: Log<S>,
    role: NodeRole,
    node_tx: UnboundedSender<Message>,
}

impl<S> RaftNode<S> where S: Store{
    pub fn new(id: String, store: S, node_tx: UnboundedSender<Message>) -> RaftNode<S> {
        RaftNode {
            id,
            peers: vec![],
            term: 0,
            log: Log::new(store),
            role: NodeRole::Follower,
            node_tx,
        }
    }

    pub fn step(&self, msg: Message) -> Result<()> {
        /*
        match self.role {
            NodeRole::Candidate => {},
            NodeRole::Follower => {},
            NodeRole::Leader => {},
        };*/
        println!("setp message: {:?}", msg);
        Ok(())
    }

    pub fn tick(mut self) -> Result<Self> {
        self.term += 1;
        //println!("node tick {:#?}", self);
        let msg = Message{term: 1, event: Event::None};
        self.node_tx.unbounded_send(msg);
        Ok(self.into())
    }
}
