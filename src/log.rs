use crate::Store;

#[derive(Debug)]
pub struct Log {
    store: Box<dyn Store>,
    pub last_index: u64,
    pub last_term: u64,
}

impl Log {
    pub fn new(store: Box<Store>) -> Log{
        Log { store, last_term:0, last_index:0 }
    }
}
