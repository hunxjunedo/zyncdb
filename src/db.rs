use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::expiry::SweeperContext;


type DbHandle = Arc<Mutex<HashMap<String, String>>>;

#[derive(Default)]
pub struct Database {
    //todo: allow non string
    pub data: DbHandle,
    pub sweeper_context: SweeperContext,
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }
}