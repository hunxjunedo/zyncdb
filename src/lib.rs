use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};
mod commands;
mod expiry;
use anyhow::{Context, Ok, Result, format_err};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpStream};
use crate::{commands::{Command, Del, Expire, Get, Set}, expiry::{Expiry, SweeperContext}};
pub struct Connection {
    address: SocketAddr,
    reader: BufReader<TcpStream>
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
        Self {  address: connection_raw.1, reader: BufReader::new(connection_raw.0) }
    }

    pub async fn read_line(&mut self) -> String {
        let mut buf = String::new();
        let bytes_read = self.reader.read_line(&mut buf).await.expect("could not read buffer data");
        println!("a total of {} bytes were read from a connection", bytes_read);
        buf
    }
}



pub async fn connection_processor(db: Arc<Database>, mut connection: Connection) {
    //spawned for each connection
    //run in a loop, read whatever data comes into a buffer
    let db = db.clone();
    loop {
        //1. read a line
        let raw_data = connection.read_line().await;
        if raw_data.len() == 0 {
            //break since connection closed, so this task will be completed
            break;
        }
        //2. parse the raw data
        let command = from_string(raw_data);
        if let Some(err) = command.as_ref().err(){
            connection.reader.get_mut().write(&err.to_string().into_bytes()).await.expect("could not send response");
            continue;
        }
        let command = command.unwrap();
        //3. process all the commands sequentially for now
        //todo: allow independent commands to progress in parallel
            
            let command_result = command.execute(&db);
            connection.reader.get_mut().write(&command_result.into_bytes()).await.expect("could not send response");
        
    }
}