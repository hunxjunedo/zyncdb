use std::{collections::HashMap};

use tokio::sync::mpsc::Receiver;

use crate::{MainMpscChannelMessage, expiry::{Expiry}};


pub type DbHandle = HashMap<String, Entry>;
pub type Entry = ( String,  Option<Expiry>);

#[derive(Default)]
pub struct Database {
    //todo: allow non string
    pub data: DbHandle,
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }
}

pub async fn db_worker(mut db: Database, mut multi_reciever: Receiver<MainMpscChannelMessage>) {
    loop {
        let (sender, command) = multi_reciever.recv().await.unwrap();
        let command_result = command.execute(&mut db);
        if let Some(sender) = sender {
            sender.send(command_result);
        }
    }
}