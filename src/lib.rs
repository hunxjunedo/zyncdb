pub mod db;
pub mod commands;
pub mod expiry;
pub mod db_operations;
pub mod background;
pub mod constants;
use std::collections::HashMap;
use std::fmt::format;
use std::str::FromStr;
use std::{
    env,
    net::SocketAddr,
    time::Duration,
};
use tokio::sync::oneshot::channel;
use tokio::sync::{oneshot, mpsc};
use tokio_stream::StreamExt;
use anyhow::{Context, Ok, Result, format_err};
use tokio::{io::AsyncWriteExt, net::TcpStream, select, sync::OwnedSemaphorePermit, time::sleep};
use tokio_util::{
    codec::{Framed, LinesCodec},
    sync::CancellationToken,
};

use crate::commands::{Command, Get, Set, Del, Expire};
use crate::db::{Database, Entry};
use crate::db_operations::DbOperation;
pub struct Connection {
    address: SocketAddr,
    framed: Framed<TcpStream, LinesCodec>,
}



pub fn read_env_var<T: FromStr>(key: &str) -> T {
    env::var(key)
        .unwrap()
        .parse::<T>()
        .unwrap_or_else(|_| panic!("environment variable not defined or has invalid value"))
}

pub fn from_string(raw_data: String) -> Result<Box<dyn Command + Send>> {
    let commands_chunks: Vec<&str> = raw_data.split(" ").collect();
    println!("{:?}", commands_chunks);
    //todo: command should decide itself what data to get
    match commands_chunks[0] {
        "GET" => {
            return Ok(Box::new(Get {
                field: commands_chunks[1].to_string(),
            }) as Box<dyn Command + Send>);
        }
        "SET" => {
            return Ok(Box::new(Set {
                field: commands_chunks[1].to_string(),
                data: commands_chunks[2].to_string(),
                ttl: commands_chunks[3].parse::<u64>().map_or(None, |v|Some(v))
            }));
        }
        "DEL" => {
            return Ok(Box::new(Del {
                field: commands_chunks[1].to_string(),
            }));
        }
        "EXP" => {
            return Ok(Box::new(Expire {
                field: commands_chunks[1].to_string(),
                ttl: commands_chunks[2]
                    .parse::<u64>()
                    .context("please provide a positive number")?,
            }));
        }
        _ => return Err(format_err!("invalid command")),
    }
}

#[derive(Debug)]
pub enum ResponseFromDbOp {
    NormalResponse(String),
    Nil,
    Error(String)
}
type ResponseSender = oneshot::Sender<ResponseFromDbOp>;
pub type MainMpscChannelMessage = (Option<ResponseSender>, Box<dyn DbOperation + Send>);


impl Connection {
    pub fn new(connection_raw: (TcpStream, SocketAddr)) -> Self {
        Self {
            address: connection_raw.1,
            framed: Framed::new(
                connection_raw.0,
                LinesCodec::new_with_max_length(read_env_var("MAX_COMMAND_LENGTH")),
            ),
        }
    }
}

pub async fn connection_processor(
    mut connection: Connection,
    cancellation_token: CancellationToken,
    _permit: OwnedSemaphorePermit,
    multi_sender: mpsc::Sender<MainMpscChannelMessage>
) {
    //spawned for each connection
    //run in a loop, read whatever data comes into a buffer
    loop {
        //todo: use decoder to return final command instead of string
        //1. read a line
        //cancel called or line read, whiever happens early
        let raw_data = select! {
             raw_data = connection.framed.next() => raw_data,
             _ = sleep(Duration::from_secs(read_env_var("CONNECTION_TIMEOUT"))) => {
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
            connection
                .framed
                .get_mut()
                .write(&("command very large\n").to_string().into_bytes())
                .await
                .expect("could not send response");
            continue;
        };
        //2. parse the raw data
        let raw_data = raw_data.unwrap();
        let command = from_string(raw_data);
        if let Some(err) = command.as_ref().err() {
            connection
                .framed
                .get_mut()
                .write(&format!("ERROR: {} \n", err).into_bytes())
                .await
                .expect("could not send response");
            continue;
        }
        let command = command.unwrap().execute(); //.execute() returns db_operation for now
        //3. process all the commands sequentially for now
        //todo: allow independent commands to progress in parallel

        //now: start a one shot channel, and send it over with the command that needs to be executed
        let (sender, reciever) = channel::<ResponseFromDbOp>();
        multi_sender.send((Some(sender), command)).await;
        let command_result = reciever.await.unwrap();
        let to_write = match command_result {
            ResponseFromDbOp::NormalResponse(result) => {
                result
            },
            ResponseFromDbOp::Error(e) => {
                format!("ERROR: {}", e)
            },
            _ => format!("SOMETHING WENT WRONG")
        };

            connection
            .framed
            .get_mut()
            .write(&(to_write + "\n").into_bytes())
            .await
            .expect("could not send response");
  
    }
}
