use std::collections::HashMap;
use anyhow::Result;
use std::ops::Bound;
use crate::log::{Scan, Store, Range};

#[derive(Debug)]
pub struct MemoryStore {
    log: Vec<Vec<u8>>,
    committed: u64,
    metadata: HashMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore { log: vec![], committed: 0, metadata: HashMap::new() }
    }
}

impl Store for MemoryStore {
    fn set_metadata(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.metadata.insert(key, value).unwrap();
        Ok(())
    }

    fn get_metadata(&self, key: Vec<u8>) -> Result<Vec<u8>> {
        self.metadata.get(&key).map(|v|v.to_vec()).ok_or(anyhow::anyhow!("no key"))
    }

    fn get(&self, index: u64) -> Result<Option<Vec<u8>>> {
        if index == 0 {
            Ok(None)
        } else {
            let v = self.log.get(index as usize - 1).unwrap();
            Ok(Some(v.clone()))
        }
    }

    fn append(&mut self, entry: Vec<u8>) -> Result<u64> {
        self.log.push(entry);
        Ok(self.log.len() as u64)
    }

    fn scan(&self, range: Range) -> Scan {
        Box::new(
            self.log
                .iter()
                .take(match range.end {
                    Bound::Included(n) => n as usize,
                    Bound::Excluded(0) => 0,
                    Bound::Excluded(n) => n as usize - 1,
                    Bound::Unbounded => std::usize::MAX,
                })
                .skip(match range.start {
                    Bound::Included(0) => 0,
                    Bound::Included(n) => n as usize - 1,
                    Bound::Excluded(n) => n as usize,
                    Bound::Unbounded => 0,
                })
                .cloned()
                .map(Ok),
        )
    }

    fn commit(&mut self, index: u64) -> Result<()> {
        if index == self.committed {
            self.committed = index;
        } else {
            return Err(anyhow::anyhow!(
            format!("commit failure index:{}, commited:{}", index, self.committed))
            );
        }
        Ok(())
    }

    fn committed(&self) -> Result<u64> {
        Ok(self.committed)
    }
}