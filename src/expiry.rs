use std::{collections::HashMap, sync::{Arc, Mutex}};

use tokio::{time::Instant};

use crate::Database;

pub struct Expiry {
    pub ttl: u64,
    pub now: Instant
}

pub type SweeperContext = Arc<Mutex<HashMap<String, Expiry>>>;

impl Database {
    //returns true if is a new expiry added
    pub fn set_expiry (&self, field: String, expiry: Expiry) -> bool {
        self.sweeper_context.lock().unwrap().insert(field, expiry).is_none()
    }
}

