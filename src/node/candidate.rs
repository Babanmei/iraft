use crate::node::{RoleNode, Node, ELECTION_TIMEOUT_MIN, ELECTION_TIMEOUT_MAX};
use anyhow::Result;
use crate::message::{Message, Address, Event};
use crate::node::follower::Follower;
use crate::node::leader::Leader;
use rand::Rng;

#[derive(Debug)]
pub struct Candidate {
    election_ticks: u64,
    election_timeout: u64,

    //拉到票总数
    votes_count: u64,
}

impl Candidate {
    pub fn new() -> Candidate {
        Candidate {
            election_ticks: 0,
            election_timeout: rand::thread_rng().gen_range(
                ELECTION_TIMEOUT_MIN..=ELECTION_TIMEOUT_MAX
            ),
            votes_count: 1,//把自己一票加上先
        }
    }
}

impl RoleNode<Candidate> {
    pub fn step(mut self, msg: Message) -> Result<Node> {
        //如果新消息term > 自己term, 说明有其他候选者节点在先,
        //这时候就不要竞争了,主动退让,让世界更和谐
        if msg.term > self.term {
            if let Address::Peer(from) = &msg.from {
                return self.transfer_follower(msg.term, from.into())?.step(msg);
            }
        }

        match msg.event {
            Event::GrantVote => {
                self.role.votes_count += 1;
                if self.role.votes_count >= self.watershed() {
                    let (lt, li, term) = (self.log.last_term, self.log.last_index, self.term);
                    let mut node = self.transfer_leader()?;
                    node = node.step(Message {
                        term,
                        from: Address::Local,
                        to: Address::Peers,
                        event: Event::Heartbeat {
                            commit_index: li,
                            commit_term: lt,
                        },
                    })?;
                    return Ok(node);
                }
            }

            Event::Heartbeat{..} => {
                if let Address::Peer(from) = &msg.from {
                    return self.transfer_follower(msg.term, from.clone())?.step(msg);
                }
            }
            _ => {}
        }

        Ok(Node::Candidate(self.into()))
    }

    pub fn tick(mut self) -> Result<Node> {
        println!("candidate tick");
        self.role.election_ticks += 1;
        if self.role.election_ticks >= self.role.election_timeout {
            self.term += 1;
            self.role = Candidate::new();
            self.send(Address::Peers, Event::SolicitVote {
                last_term: self.log.last_term,
                last_index: self.log.last_index,
            })?;
            Ok(Node::Candidate(self))
        } else {
            Ok(Node::Candidate(self))
        }
    }
}

impl RoleNode<Candidate> {
    fn transfer_follower(mut self, term: u64, leader: String) -> Result<RoleNode<Follower>> {
        self.role.election_ticks = 0;
        let mut node = self.transfer_role(Follower::new(Some(leader), None))?;
        node.term = term;
        //log save
        //abort all request
        Ok(node)
    }

    fn transfer_leader(mut self) -> Result<Node> {
        let mut node = self.transfer_role(super::Leader::new())?;
        Ok(Node::Leader(node.into()))
    }
}
