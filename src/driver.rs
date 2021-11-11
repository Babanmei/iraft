use anyhow::Result;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use crate::message::Address;

/// 状态机
/// 针对业务层而来的command, 状态机会执行一系列操作
/// 此接口规范针对业务层的command的执行规范
///
/// 此接口提出raft下的所有指令中针对业务层面的, 我们可以针对自己业务实现它
pub trait State: Send {
    fn applied_index(&self) -> u64;
    fn mutate(&mut self, index: u64, command: Vec<u8>) -> Result<Vec<u8>>;
    fn query(&self, command: Vec<u8>) -> Result<Vec<u8>>;
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Vote { term: u64, index: u64, address: Address },
}

pub struct Driver {
    applied_index: u64,
    state_rx: UnboundedReceiver<Instruction>,
}

impl Driver {
    pub fn new(rx: UnboundedReceiver<Instruction>) -> Driver {
        Self {
            state_rx: rx,
            applied_index: 0,
        }
    }

    pub async fn drive(mut self, mut state: Box<State>) -> Result<()> {
        while let Some(instruction) = self.state_rx.next().await {
            println!("{:?}", instruction);
        }
        Ok(())
    }
}


pub struct MVCC {}

impl State for MVCC {
    fn applied_index(&self) -> u64 {
        0
    }

    fn mutate(&mut self, index: u64, command: Vec<u8>) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn query(&self, command: Vec<u8>) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}