pub mod memory_store;
pub mod log;


use std::fmt::Debug;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::ops::{Bound, RangeBounds};

///对存储层定义一个scan 返回bytes迭代器
pub type Scan<'a> = Box<dyn Iterator<Item=Result<Vec<u8>>> + 'a>;

///扫描边界
pub struct Range {
    start: Bound<u64>,
    end: Bound<u64>,
}

impl Range {
    pub fn from(range: impl RangeBounds<u64>) -> Self {
        Self {
            start: match range.start_bound() {
                Bound::Included(v) => Bound::Included(*v),
                Bound::Excluded(v) => Bound::Excluded(*v),
                Bound::Unbounded => Bound::Unbounded,
            },
            end: match range.end_bound() {
                Bound::Included(v) => Bound::Included(*v),
                Bound::Excluded(v) => Bound::Excluded(*v),
                Bound::Unbounded => Bound::Unbounded,
            },
        }
    }
}

///定义在存储上的一系列操作, 都以bytes形式操作
pub trait Store: Debug + Sync + Send {
    //元数据操作
    fn set_metadata(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn get_metadata(&self, key: Vec<u8>) -> Result<Vec<u8>>;
    //log数据操作
    fn get(&self, index: u64) -> Result<Option<Vec<u8>>>;
    fn append(&mut self, entry: Vec<u8>) -> Result<u64>;
    fn scan(&self, range: Range) -> Scan;
    //提交log和获取已提交的log index
    fn commit(&mut self, index: u64) -> Result<()>;
    fn committed(&self) -> Result<u64>;
    //fn size(&self) -> u64;
    //fn truncate(&mut self, index: u64) -> Result<u64>;
}


fn serialize<V: Serialize>(value: &V) -> Result<Vec<u8>> {
    Ok(bincode::serialize(value)?)
}

fn deserialize<'a, V: Deserialize<'a>>(bytes: &'a [u8]) -> Result<V> {
    Ok(bincode::deserialize(bytes)?)
}
