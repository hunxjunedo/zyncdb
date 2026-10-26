use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::time::Instant;

use crate::Database;

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
        self.sweeper_context
            .insert(field, expiry)
            .is_none()
    }

    //call when already deleted the value
    pub fn remove_expiry(&mut self, field: String) {
        self.sweeper_context.remove(&field);
    }
}
