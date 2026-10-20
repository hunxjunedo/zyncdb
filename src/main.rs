use std::sync::Arc;

use tokio::{net::TcpListener, spawn};
use zyncdb::{Connection, Database, connection_processor};

#[tokio::main]
async fn main() {
    //bind socket to ip
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("error: could not the listener to the address");
    let main_db_instance = Arc::new(Database::new());
    loop {
        //accept the incoming connections, one at a time
        let connection = listener.accept().await.expect("could not accept a connection");
        let connection = Connection::new(connection);
        let main_db_instance = Arc::clone(&main_db_instance);
        spawn(connection_processor(main_db_instance, connection));
        

    }
}


