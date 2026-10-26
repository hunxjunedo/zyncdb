use std::sync::Arc;

use tokio::time::Instant;

use crate::{db::{Database, DbHandle}, db_operations::{DbOperation, GetKey, RemoveKey, SetEvictionPolicy, SetKey}};


//the user facing commands
//for now just make them return a db command, until we get to more complex commands.
//todo: make Command recieve the multi-sender, so that it has more control.
pub trait Command {
    fn execute(&self) -> DbOp;
}

type DbOp = Box<dyn DbOperation + Send>;

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
    fn execute(&self) -> DbOp {
        Box::from(GetKey{key: self.field.clone()})
    }
}

impl Command for Set {
    fn execute(&self) -> DbOp {
        Box::from(SetKey{key: self.field.clone(), value: self.data.clone()})
    }
}
impl Command for Del {
    fn execute(&self) -> DbOp {
        Box::from(RemoveKey{key: self.field.clone()})
    }
}

pub struct Expire {
    pub field: String,
    pub ttl: u64,
}

impl Command for Expire {
    fn execute(&self) -> DbOp {
        Box::from(SetEvictionPolicy{key: self.field.clone(), ttl: self.ttl.clone()})
    }
}
