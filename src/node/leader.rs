use crate::node::{RoleNode, Node, HEARTBEAT_INTERVAL};
use anyhow::Result;
use crate::message::{Message, Address, Event};
use crate::driver::Instruction;
use std::collections::HashMap;
use crate::log::log::Entry;

#[derive(Debug)]
pub struct Leader {
    heartbeat_ticks: u64,
    //记录孩儿们的最后状态
    peer_next_index: HashMap<String, u64>,
    peer_last_index: HashMap<String, u64>,
}

impl Leader {
    pub fn new(peers: Vec<String>, last_index: u64) -> Leader {
        let (mut nexts, mut lasts) = (HashMap::new(), HashMap::new());
        peers.iter().for_each(|peer| {
            nexts.insert(peer.clone(), last_index + 1);
            lasts.insert(peer.clone(), 0);
        });
        Leader {
            heartbeat_ticks: 0,
            peer_next_index: nexts,
            peer_last_index: lasts,
        }
    }
}

impl RoleNode<Leader> {
    /// 超过心跳间隔 发送心跳
    pub fn tick(mut self) -> Result<Node> {
        if !self.peers.is_empty() {
            self.role.heartbeat_ticks += 1;
            //持续心跳
            if self.role.heartbeat_ticks >= HEARTBEAT_INTERVAL {
                self.role.heartbeat_ticks = 0;
                self.send(Address::Peers, Event::Heartbeat {
                    commit_index: self.log.commit_index,
                    commit_term: self.log.commit_term,
                })?;
            }
        }
        Ok(Node::Leader(self))
    }

    pub fn step(mut self, msg: Message) -> Result<Node> {
        //有人起义成功了, 不做无为抵抗
        if msg.term > self.term {
            if let Address::Peer(from) = &msg.from {
                let mut node = self.transfer_role(super::Follower::new(Some(from.clone()), None))?;
                return node.step(msg);
            }
        }

        match msg.event {
            Event::ConfirmLeader { commit_index, has_committed } => {
                if let Address::Peer(from) = msg.from.clone() {
                    self.state_sender.unbounded_send(Instruction::Vote {
                        term: msg.term,
                        index: commit_index,
                        address: msg.from,
                    })?;
                    if !has_committed {
                        self.replicate(&from)?;
                    }
                }
            }
            _ => println!("~~~"),
        }
        Ok(Node::Leader(self.into()))
    }
}

impl RoleNode<Leader> {
    ///给peer复制一条日志
    fn replicate(&self, peer: &str) -> Result<()> {
        //1, peer对应的log index
        let next_index = self.role
            .peer_next_index
            .get(peer)
            .ok_or_else(|| anyhow::anyhow!("没有这个peer"))?;
        let base_index = if *next_index > 0 { *next_index - 1 } else { 0 };
        //2, 当前term
        let base_term = match self.log.get(base_index)? {
            Some(e) => e.term,
            None if base_index == 0 => 0,
            None => return Err(anyhow::anyhow!("当前term获取错误")),
        };
        //3, 取出leader自己的这部分日志
        let entries: Vec<Entry> = self.log.scan(next_index..).collect::<Result<Vec<_>>>()?;
        //4, 发送给他
        self.send(Address::Peer(peer.to_string()), Event::ReplicateEntries {
            base_index,
            base_term,
            entries,
        })?;
        Ok(())
    }

    fn commit(&mut self) -> Result<u64> {
        Ok(0)
    }
}

