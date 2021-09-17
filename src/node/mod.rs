use anyhow::Result;
use futures::channel::mpsc::UnboundedSender;

use crate::log::Log;
use crate::memory_store::MemoryStore;
use crate::message::{Address, Event, Message};
use crate::node::candidate::Candidate;
use crate::node::follower::Follower;
use crate::node::leader::Leader;
use crate::Store;

pub mod leader;
pub mod follower;
pub mod candidate;


/// 心跳间隔
const HEARTBEAT_INTERVAL: u64 = 1;
/// 选举超时时间间隔 MIN - MAX
const ELECTION_TIMEOUT_MIN: u64 = 2 * HEARTBEAT_INTERVAL;
const ELECTION_TIMEOUT_MAX: u64 = 5 * HEARTBEAT_INTERVAL;


#[derive(Debug)]
pub enum Node {
    Candidate(RoleNode<Candidate>),
    Follower(RoleNode<Follower>),
    Leader(RoleNode<Leader>),
}

impl Node {
    pub async fn new(id: String, log: Log, peers: Vec<String>, tx: UnboundedSender<Message>) -> Result<Node> {
        let n = RoleNode { id, log, peers, term: 0, to_peer_tx: tx, role: Follower::new(None, None) };
        Ok(Node::Follower(n))
    }

    pub fn tick(mut self) -> Result<Node> {
        match self {
            Node::Follower(f) => f.tick(),
            Node::Leader(l) => l.tick(),
            Node::Candidate(c) => c.tick(),
        }
    }

    pub fn step(mut self, msg: Message) -> Result<Node> {
        match self {
            Node::Follower(f) => f.step(msg),
            Node::Leader(l) => l.step(msg),
            Node::Candidate(c) => c.step(msg),
        }
    }
}

#[derive(Debug)]
pub struct RoleNode<Role> {
    id: String,
    log: Log,
    peers: Vec<String>,
    term: u64,
    to_peer_tx: UnboundedSender<Message>,
    role: Role,
}

impl<Role> RoleNode<Role> {
    pub fn transfer_role<R>(self, r: R) -> Result<RoleNode<R>> {
        Ok(RoleNode {
            id: self.id,
            log: self.log,
            peers: self.peers,
            term: self.term,
            to_peer_tx: self.to_peer_tx,
            role: r,
        })
    }

    pub fn send(&self, to: Address, event: Event) -> Result<()> {
        let msg = Message {
            term: self.term,
            from: Address::Local,
            to,
            event,
        };
        self.to_peer_tx.unbounded_send(msg);
        Ok(())
    }

    ///超过这个数的人赞成, 恭喜你,你就当选了
    pub fn watershed(&self) -> u64 {
        (self.peers.len() as u64 + 1) / 2 + 1
    }
}
