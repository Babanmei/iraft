use anyhow::Result;
use std::path::Iter;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::ops::{Bound, RangeBounds};
use crate::log::{Store, serialize, deserialize, Range};


#[derive(Clone, Debug, PartialEq)]
pub struct MetadateKey;

impl MetadateKey {
    fn encode(&self) -> Vec<u8> {
        vec![0x00]
    }
}

///一条log的结构
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub index: u64,
    pub term: u64,
    pub command: Option<Vec<u8>>,
}

///定义面向业务的log操作
#[derive(Debug)]
pub struct Log {
    pub(crate) store: Box<dyn Store>,
    pub(crate) last_index: u64,
    pub(crate) last_term: u64,

    pub(crate) commit_index: u64,
    pub(crate) commit_term: u64,
}

pub type Scan<'a> = Box<Iterator<Item=Result<Entry>> + 'a>;

impl Log {
    pub fn new(store: Box<Store>) -> Log {
        Log {
            store,
            last_term: 0,
            last_index: 0,
            commit_index: 0,
            commit_term: 0,
        }
    }

    ///元数据的get / set
    pub fn save_metadata(&mut self, term: u64, voted_for: Option<&str>) -> Result<()> {
        self.store.set_metadata(MetadateKey.encode(), serialize(&(term, voted_for))?)
    }

    pub fn get_metadata(&self) -> Result<(u64, Option<String>)> {
        let value = self.store.get_metadata(MetadateKey.encode())?;
        Ok(deserialize(&value[..])?)
    }

    ///数据的 get / set
    pub fn get(&self, index: u64) -> Result<Option<Entry>> {
        self.store.get(index)?.map(|v| deserialize(&v)).transpose()
    }
    pub fn append(&mut self, term: u64, command: Option<Vec<u8>>) -> Result<Entry> {
        let entry = Entry { index: self.last_index + 1, term, command };
        let _ = self.store.append(serialize(&entry)?)?;
        self.last_index = entry.index;
        self.last_term = term;
        Ok(entry)
    }
    pub fn scan(&self, range: impl RangeBounds<u64>) -> Scan {
        Box::new(self.store.scan(Range::from(range)).map(|r| r.and_then(|v| deserialize(&v))))
    }

    /// 提交日志
    pub fn commit(&mut self, index: u64) -> Result<u64> {
        match self.get(index)? {
            Some(entry) => {
                self.store.commit(index)?;
                self.commit_index = entry.index;
                self.commit_term = entry.term;
                Ok(index)
            }
            None => Err(anyhow::anyhow!(format!("提交index:{}错误", index))),
        }
    }

    pub fn has(&self, index: u64, term: u64) -> Result<bool> {
        match self.get(index)? {
            Some(e) => Ok(e.term == term),
            None if index == 0 && term == 0 => Ok(true),
            None => Ok(false),
        }
    }
}
