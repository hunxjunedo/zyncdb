use std::{collections::HashMap};

use tokio::sync::mpsc::Receiver;

use crate::{expiry::SweeperContext, main_mpsc_channel_message};


pub type DbHandle = HashMap<String, String>;

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

pub async fn db_worker(mut db: DbHandle, mut multi_reciever: Receiver<main_mpsc_channel_message>) {
    loop {
        let (sender, command) = multi_reciever.recv().await.unwrap();
        let command_result = command.execute(&mut db);
        sender.send(command_result);
    }
}