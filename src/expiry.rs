use std::{collections::HashMap, time::Duration};

use tokio::time::Instant;

use crate::{Database, Entry, db_operations::{DbOperation, RemoveKey}, expiry};

#[derive(Clone)]
pub struct Expiry {
    pub ttl: u64,
    pub now: Instant,
}

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

    pub fn key_expired(entry: &Entry) -> bool {
        let expiry = entry.1.clone();
        match expiry {
            Some(expiry) => return expiry.now.elapsed() >= Duration::from_secs(expiry.ttl),
            None => false,
        }
    }

    pub fn get_expired_keys(&self, limit: usize) -> Vec<String> {
        self.data
            .iter()
            .filter(|v| Self::key_expired(v.1))
            .take(limit)
            .map(|(key, _)| key.clone())
            .collect()
    }

    pub fn get_key_if_active(&mut self, key: String) -> Option<Entry> {
        //use this instead of directly reading a key
        let Some(value) = self.data.get(&key).cloned() else {
            return None;
        };
        if Self::key_expired(&value) {
            //remove the key as it has expired
            RemoveKey{key}.execute(self);
            None
        } else {
            Some(value)
        }
    }
}
