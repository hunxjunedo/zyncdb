use std::{env, sync::Arc};
mod background;
use tokio::{ join, net::TcpListener, select, spawn, sync::{Semaphore}, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use zyncdb::{Connection, Database, connection_processor, read_env_var};
use dotenv::dotenv;

use crate::background::{bg_sweeper, shutdown_listener};

#[tokio::main]
async fn main() {
    dotenv().ok();
    //bind socket to ip
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("error: could not the listener to the address");
    let main_db_instance = Arc::new(Database::new());
    //note: everytime a vec has to increase size, it doubles. remember might need for mem optimization
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let semaphore = Arc::new(Semaphore::new(read_env_var("CONNECTION_LIMIT")));
    //spawn the bg tasks, like sweeper
    let token = CancellationToken::new();
    spawn(bg_sweeper(main_db_instance.clone()));
    spawn(shutdown_listener(token.clone()));
    loop {
        //accept the incoming connections, one at a time
        let connection = select! {
            connection = listener.accept() => connection,
            _ = token.cancelled() => break
        }.expect("could not accept a connection");
        //this will stop accepting connections once cancel is called
        let token = CancellationToken::clone(&token);
        let connection: Connection = Connection::new(connection);
        let main_db_instance = Arc::clone(&main_db_instance);
        //attempt to acquire the semaphore here: if its ready you spawn the task, if not you don't accept any more connections
        //ordinary semaphorepermit borrows semaphore. so can't.
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let handle = spawn(connection_processor(main_db_instance, connection, token, permit));
        handles.push(handle);
        
    }
    println!("loop broken because cancel called");
    //imp: awaiting sequentially does not mean we are progressing them sequentially, tokio is already driving them. Time taken for this loop to complete will still be the max. time taken by a task;
    for handle in handles {
        handle.await;
    };
}


