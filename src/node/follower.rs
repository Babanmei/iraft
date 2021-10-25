use anyhow::Result;
use futures::channel::mpsc::UnboundedSender;
use rand::Rng;

use crate::message::{Event, Message, Address};
use super::{Candidate, Node, RoleNode};
use crate::node::{ELECTION_TIMEOUT_MIN, ELECTION_TIMEOUT_MAX};


#[derive(Debug)]
pub struct Follower {
    leader: Option<String>,
    voted_for: Option<String>,
    //选举计时器
    leader_seen_ticks: u64,
    leader_seen_timeout: u64,
}

impl Follower {
    pub fn new(leader: Option<String>, voted_for: Option<String>) -> Follower {
        Follower {
            leader,
            voted_for,
            leader_seen_ticks: 0,
            leader_seen_timeout: rand::thread_rng().gen_range(
                ELECTION_TIMEOUT_MIN..=ELECTION_TIMEOUT_MAX
            ),
        }
    }
}

impl RoleNode<Follower> {
    pub fn tick(mut self) -> Result<Node> {
        //选举:
        // 等待超过随机时间时,将term加1(准备开始一个新任期),角色转换为候选者,并向所有节点发送'拉票'事件
        self.role.leader_seen_ticks += 1;
        if self.role.leader_seen_ticks >= self.role.leader_seen_timeout {
            let (li, lt) = ((&self.log).last_index.clone(), (&self.log).last_term);
            let mut node = self.transfer_role(Candidate::new())?;
            node.term += 1;
            node.log.save_metadata(node.term, None)?;
            node.send(Address::Peers, Event::SolicitVote {
                last_index: li,
                last_term: lt,
            });
            Ok(Node::Candidate(node.into()))
        } else {
            Ok(Node::Follower(self))
        }
    }

    pub fn step(mut self, msg: Message) -> Result<Node> {
        //1, 如果msg.term > self.term: 说明是新一届的消息, 自己还follower(保存log,拒绝其他节点请求)
        if msg.term > self.term || self.role.leader.is_none() {
            self.term = msg.term;
            self.log.save_metadata(self.term, None)?;
            if let Address::Peer(ref from) = &msg.from {
                self.role.leader = Some(from.clone());
                return Node::Follower(self.into()).step(msg);
            }
        }

        //2, 如果msg.from是自己已经承认的leader(msg.form==self.voted_for), 选举计时器清零(就不tick了)
        if let Address::Peer(from) = &msg.from {
            let x = &(&self).role;
            if Some(from) == x.voted_for.as_ref() {
                self.role.leader_seen_ticks = 0;
            }
        }

        //处理消息
        match msg.event {
            Event::Heartbeat { commit_index, commit_term } => {
                //todo some
                println!("认主成功, 心跳加速..");
                self.send(msg.from, Event::ConfirmLeader {
                    commit_index,
                    has_committed: true,
                })?;
            }
            Event::SolicitVote { last_index, last_term } => {
                //处理拉票请求
                //1, 如果msg.form不是自己已经认定的leader, 不予搭理
                if let Some(voted_for) = &self.role.voted_for {
                    if msg.from != Address::Peer(voted_for.clone()) {
                        return Ok(Node::Follower(self.into()));
                    }
                }
                //2, msg.last_term < 自己term , 不予搭理
                if last_term < self.log.last_term {
                    return Ok(Node::Follower(self.into()));
                }
                //3, term相等 并且 index < 自己的, 不予搭理
                if last_term < self.log.last_term && last_index < self.log.last_index {
                    return Ok(Node::Follower(self.into()));
                }
                //4, 发送赞成消息
                self.send((&msg.from).clone(), Event::GrantVote)?;
                if let Address::Peer(from) = &msg.from {
                    self.role.voted_for = Some(from.clone());
                    self.log.save_metadata(self.term, Some(&from))?
                }
            }
            _ => (),
        }

        Ok(Node::Follower(self.into()))
    }
}