use std::{collections::HashMap, time::Duration};

use tokio::time::Instant;

use crate::{Database, Entry, expiry};

#[derive(Clone)]
pub struct Expiry {
    pub ttl: u64,
    pub now: Instant,
}

pub type SweeperContext = HashMap<String, Expiry>;

impl Database {
    //these functions will be called by the actor, with &mut db
    //returns true if is a new expiry added
    pub fn set_expiry(&mut self, field: String, expiry: Expiry) -> bool {
        let mut val = self.data.remove(&field).unwrap();
        let already_expiry_set = val.1.is_some();
        val.1 = Some(expiry);
        self.data.insert(field, val);
        return already_expiry_set;
    }

    pub fn get_expired_keys(&self, limit: usize) -> Vec<String> {
        self.data
            .iter()
            .filter(|(_, (_, expiry))| match expiry {
                Some(expiry) => return expiry.now.elapsed() >= Duration::from_secs(expiry.ttl),
                None => false,
            })
            .take(limit)
            .map(|(key, _)| key.clone())
            .collect()
    }
}
