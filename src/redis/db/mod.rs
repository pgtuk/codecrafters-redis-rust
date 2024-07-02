use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use base64::prelude::*;
use base64::prelude::BASE64_STANDARD;
use bytes::Bytes;
use tokio::sync::Notify;
use tokio::time::{Duration, Instant, sleep_until};

const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

#[derive(Debug, Clone)]
pub struct Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,

    notify_expire: Notify,
}

#[derive(Debug)]
struct State {
    entries: HashMap<String, Entry>,

    shutdown: bool,
    // track TTLs
    expirations: BTreeSet<(Instant, String)>,
}

#[derive(Debug)]
struct Entry {
    data: Bytes,

    expires_at: Option<Instant>,
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
                shutdown: false,
                expirations: BTreeSet::new(),
            }),
            notify_expire: Notify::new(),
        });

        tokio::spawn(remove_expired_tasks(shared.clone()));

        Db { shared }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.state.lock().unwrap();

        state.entries.get(key).map(|entry| entry.data.clone())
    }

    pub fn set(&mut self, key: String, data: Bytes, expire: Option<Duration>) {
        let mut state = self.shared.state.lock().unwrap();
        let mut notify = false;

        let expires_at = expire.map(|duration| {
            let expire = Instant::now() + duration;
            notify = state.expirations.first().map(|first_expire|
            first_expire.0 > expire
            ).unwrap_or(true);

            expire
        });

        let old_entry = state.entries.insert(key.clone(), Entry { data, expires_at });

        if let Some(old_val) = old_entry {
            if let Some(expire) = old_val.expires_at {
                state.expirations.remove(&(expire, key.clone()));
            }
        }

        if let Some(expire) = expires_at {
            state.expirations.insert((expire, key));
        };

        drop(state);

        if notify {
            self.shared.notify_expire.notify_one();
        }
    }

    pub fn build_rdb_frame(&self) -> Vec<u8> {
        BASE64_STANDARD.decode(EMPTY_RDB).unwrap()
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        let mut state = self.shared.state.lock().unwrap();
        state.shutdown = true;

        drop(state);
        self.shared.notify_expire.notify_one();
    }
}

impl Shared {
    fn remove_expired(&self) -> Option<Instant> {
        let mut state = self.state.lock().unwrap();
        let state = &mut *state;

        if state.shutdown {
            return None;
        }

        let now = Instant::now();

        while let Some(&(expire, ref key)) = state.expirations.iter().next() {
            if expire > now {
                return Some(expire);
            }

            state.entries.remove(key);
            state.expirations.remove(&(expire, key.clone()));
        }

        None
    }

    fn is_up(&self) -> bool {
        !self.state.lock().unwrap().shutdown
    }
}

async fn remove_expired_tasks(shared: Arc<Shared>) {
    while shared.is_up() {
        if let Some(expire) = shared.remove_expired() {
            tokio::select! {
                _ = sleep_until(expire) => {},
                _ = shared.notify_expire.notified() => {}
            }
        } else {
            shared.notify_expire.notified().await;
        }
    }
}

#[cfg(test)]
mod tests;
