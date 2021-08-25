use crate::Store;
use serde_derive::{Deserialize, Serialize};


#[derive(Clone, Debug)]
pub struct Log<S: Store> {
    store: S,
}

impl<S> Log<S> where S: Store {
    pub fn new(store: S) -> Log<S> {
        Log { store }
    }
}
