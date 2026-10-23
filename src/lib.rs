use std::{collections::HashMap, net::SocketAddr, os::raw, sync::{Arc, Mutex}, time::Duration};
use tokio_stream::{StreamExt};
mod commands;
mod expiry;
use anyhow::{Context, Ok, Result, format_err};
use tokio::{io::AsyncWriteExt, net::TcpStream, select, sync::{OwnedSemaphorePermit, SemaphorePermit}, time::sleep};
use tokio_util::{codec::{Framed, LinesCodec}, sync::CancellationToken};
use crate::{commands::{Command, Del, Expire, Get, Set}, expiry::{Expiry, SweeperContext}};
pub struct Connection {
    address: SocketAddr,
    framed: Framed<TcpStream, LinesCodec>
}

type DbHandle = Arc<Mutex<HashMap<String, String>>>;


#[derive(Default)]
pub struct Database {
    //todo: allow non string
    pub data: DbHandle,
    pub sweeper_context: SweeperContext
}


    pub fn from_string(raw_data: String) -> Result<Box<dyn Command + Send>> {
        let commands_chunks : Vec<&str> = raw_data.split(" ").collect();
            println!("{:?}", commands_chunks);
            //todo: command should decide itself what data to get
            match commands_chunks[0] {
                "GET" => return Ok(Box::new(Get {field: commands_chunks[1].to_string()}) as Box<dyn Command + Send>),
                "SET" => return Ok(Box::new(Set { field: commands_chunks[1].to_string(), data: commands_chunks[2].to_string()})),
                "DEL" => return Ok(Box::new(Del { field:  commands_chunks[1].to_string()})),
                "EXP" => return Ok(Box::new(Expire {field: commands_chunks[1].to_string(), ttl: commands_chunks[2].parse::<u64>().context("please provide a positive number")?})),
                _ => return Err(format_err!("invalid command"))
            }
        
    }

impl Database {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Connection {
    pub fn new(connection_raw: (TcpStream, SocketAddr)) -> Self {
        Self {  address: connection_raw.1, framed: Framed::new(connection_raw.0, LinesCodec::new_with_max_length(1000))}
    }
}



pub async fn connection_processor(db: Arc<Database>, mut connection: Connection, cancellation_token: CancellationToken, permit: OwnedSemaphorePermit) {
    //spawned for each connection
    //run in a loop, read whatever data comes into a buffer
    let db = db.clone();
    loop {
        //todo: use decoder to return final command instead of string
        //1. read a line
        //cancel called or line read, whiever happens early
        let raw_data = select! {
             raw_data = connection.framed.next() => raw_data,
             _ = sleep(Duration::from_secs(30)) => {
                println!("read line operation timed out. Either no command arrived for too long or command was too slow to arrive fully");
                break;
             },
             _ = cancellation_token.cancelled() => {
                println!("cancellation wins, breaking");
                break;
             }

        };
        if raw_data.is_none() {
            //break since connection closed, so this task will be completed
            break;
        }
        let raw_data = raw_data.unwrap();
        //handle long line error
        if raw_data.is_err() {
            connection.framed.get_mut().write(&("command very large").to_string().into_bytes()).await.expect("could not send response");
            continue;
        };
        //2. parse the raw data
        let raw_data = raw_data.unwrap();
        let command = from_string(raw_data);
        if let Some(err) = command.as_ref().err(){
            connection.framed.get_mut().write(&err.to_string().into_bytes()).await.expect("could not send response");
            continue;
        }
        let command = command.unwrap();
        //3. process all the commands sequentially for now
        //todo: allow independent commands to progress in parallel
            
            let command_result = command.execute(&db);
            connection.framed.get_mut().write(&command_result.into_bytes()).await.expect("could not send response");
        
    }
}