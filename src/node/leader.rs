use crate::node::{RoleNode, Node, HEARTBEAT_INTERVAL};
use anyhow::Result;
use crate::message::{Message, Address, Event};

#[derive(Debug)]
pub struct Leader {
    heartbeat_ticks: u64,
}

impl Leader {
    pub fn new() -> Leader {
        Leader { heartbeat_ticks: 0 }
    }
}

impl RoleNode<Leader> {
    /// 超过心跳间隔 发送心跳
    pub fn tick(mut self) -> Result<Node> {
        println!("leader tick");
        if !self.peers.is_empty() {
            self.role.heartbeat_ticks += 1;
            if self.role.heartbeat_ticks >= HEARTBEAT_INTERVAL {
                self.role.heartbeat_ticks = 0;
                self.send(Address::Peers, Event::Heartbeat {
                    commit_index: 0,
                    commit_term: 0,
                })?;
            }
        }
        Ok(Node::Leader(self))
    }

    pub fn step(mut self, msg: Message) -> Result<Node> {
        if msg.term > self.term {
            if let Address::Peer(from) = &msg.from {
                let mut node = self.transfer_role(super::Follower::new(Some(from.clone()), None))?;
                return node.step(msg);
            }
        }

        match msg.event {
            Event::ConfirmLeader { commit_index, has_committed } => {
                println!("从{:?}收到确认:{},{}", msg.from, commit_index, has_committed);
            }
            _ => println!("~~~"),
        }
        Ok(Node::Leader(self.into()))
    }
}