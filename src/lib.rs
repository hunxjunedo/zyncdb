use std::{collections::HashMap, net::SocketAddr, os::raw, sync::{Arc, Mutex}};
mod commands;
use tokio::{io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}, net::TcpStream, time::Instant};

use crate::commands::{Command, Get, Set, Del};
pub struct Connection {
    stream: TcpStream,
    address: SocketAddr
}

type DbHandle = Arc<Mutex<HashMap<String, String>>>;
type SweeperContext = Arc<Mutex<HashMap<String, Expiry>>>;

#[derive(Default)]
pub struct Database {
    //todo: allow non string
    pub data: DbHandle,
    pub sweeper_context: SweeperContext
}

impl Database {
    //returns true if is a new expiry added
    pub fn set_expiry (&self, field: String, expiry: Expiry) -> bool {
        self.sweeper_context.lock().unwrap().insert(field, expiry).is_none()
    }
}

pub struct Expiry {
    pub ttl: u64,
    pub now: Instant
}


    pub fn from_string(raw_data: String) -> Box<dyn Command + Send> {
        let commands_chunks : Vec<&str> = raw_data.split(" ").collect();
            println!("{:?}", commands_chunks);
            //todo: command should decide itself what data to get
            match commands_chunks[0] {
                "GET" => return Box::new(Get {field: commands_chunks[1].to_string()}) as Box<dyn Command + Send>,
                "SET" => return Box::new(Set { field: commands_chunks[1].to_string(), data: commands_chunks[2].to_string()}),
                "DEL" => return Box::new(Del { field:  commands_chunks[1].to_string()}),
                _ => panic!("unknown command!")
            }
        
    }

impl Database {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Connection {
    pub fn new(connection_raw: (TcpStream, SocketAddr)) -> Self {
        Self { stream: connection_raw.0, address: connection_raw.1 }
    }

    pub async fn read_line(&mut self) -> String {
        let mut buf = String::new();
        let mut reader = BufReader::new(&mut self.stream);
        let bytes_read = reader.read_line(&mut buf).await.expect("could not read buffer data");
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
        //2. parse the raw data
        let command = from_string(raw_data);
        //3. process all the commands sequentially for now
        //todo: allow independent commands to progress in parallel
            
            let command_result = command.execute(&db);
            connection.stream.write(&command_result.into_bytes()).await.expect("could not send response");
        
    }
}