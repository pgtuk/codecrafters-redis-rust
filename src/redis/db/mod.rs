use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


#[derive(Debug, Clone)]
pub struct Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    entries: HashMap<String, Entry>
}

#[derive(Debug)]
struct Entry {
    data: Bytes,
}

impl Db {
    pub fn new() -> Db {
        Db { 
            shared: Arc::new(Shared {
                state: Mutex::new(State { 
                    entries: HashMap::new(),
                }),
            }),
        }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.state.lock().unwrap();

        state.entries.get(key).map(|entry| entry.data.clone())
    }

    pub fn set(&mut self, key: String, value: Bytes) {
        let mut state = self.shared.state.lock().unwrap();

        state.entries.insert(key, Entry { data: value } );
    }
}
