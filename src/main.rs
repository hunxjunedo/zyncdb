use std::sync::Arc;
mod background;
use tokio::{net::TcpListener, spawn};
use tokio_util::sync::CancellationToken;
use zyncdb::{Connection, Database, connection_processor};

use crate::background::{bg_sweeper, shutdown_listener};

#[tokio::main]
async fn main() {
    //bind socket to ip
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("error: could not the listener to the address");
    let main_db_instance = Arc::new(Database::new());
    //spawn the bg tasks, like sweeper
    let token = CancellationToken::new();
    spawn(bg_sweeper(main_db_instance.clone()));
    spawn(shutdown_listener(token.clone()));
    loop {
        //accept the incoming connections, one at a time
        let connection = listener.accept().await.expect("could not accept a connection");
        let token = CancellationToken::clone(&token);
        let connection: Connection = Connection::new(connection);
        let main_db_instance = Arc::clone(&main_db_instance);
        spawn(connection_processor(main_db_instance, connection, token));
        
    }
}


