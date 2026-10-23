use std::sync::Arc;

use tokio::time::Instant;

use crate::{db::Database, expiry::Expiry};


pub trait Command {
    fn execute(&self, db: &Arc<Database>) -> String;
}

pub struct Get {
    pub field: String,
}
pub struct Set {
    pub field: String,
    pub data: String,
}
pub struct Del {
    pub field: String,
}

impl Command for Get {
    fn execute(&self, db: &Arc<Database>) -> String {
        match db.data.lock().unwrap().get(&self.field) {
            Some(value) => value.to_owned(),
            None => "Value does not exist!".to_string(),
        }
    }
}

impl Command for Set {
    fn execute(&self, db: &Arc<Database>) -> String {
        match db
            .data
            .lock()
            .unwrap()
            .insert(self.field.clone(), self.data.clone())
        {
            Some(_) => "Value update".to_string(),
            None => "Field did not exist, but created it".to_string(),
        }
    }
}
impl Command for Del {
    fn execute(&self, db: &Arc<Database>) -> String {
        match db.data.lock().unwrap().remove(&self.field) {
            Some(_) => "Field Succesfully Removed".to_string(),
            None => "Field does not exist!".to_string(),
        }
    }
}

pub struct Expire {
    pub field: String,
    pub ttl: u64,
}

impl Command for Expire {
    fn execute(&self, db: &Arc<Database>) -> String {
        let expiry = Expiry {
            ttl: self.ttl,
            now: Instant::now(),
        };
        if db.set_expiry(self.field.clone(), expiry) {
            String::from("TTL added")
        } else {
            String::from("TTL updated")
        }
    }
}
