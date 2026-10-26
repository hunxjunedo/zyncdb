use std::{time::Duration};
use tokio::{
    signal, sync::{mpsc, oneshot}, time::sleep,
};
use tokio_util::sync::CancellationToken;

use crate::{MainMpscChannelMessage, ResponseFromDbOp,  db_operations::{DbOperation, FilterKeysInSweeperContext, RemoveKey}, expiry::Expiry, read_env_var};

pub async fn bg_sweeper(multi_sender: mpsc::Sender<MainMpscChannelMessage>) {
    //responsible for checking the expiry
    //todo: use channels instead of periodically polling

    loop {
        sleep(Duration::from_secs(read_env_var("SWEEPER_FREQUENCY"))).await;
        //wake up after atleast 1 secs everytime
        //check all the expiries
        let (tx, rc) = oneshot::channel();
        let predicate: Box<dyn Fn((&String, &Expiry)) -> bool + Send> = Box::from(|(_, expiry): (&String, &Expiry)| expiry.now.elapsed() >= Duration::from_secs(expiry.ttl));
        let command =   FilterKeysInSweeperContext{
            predicate
        };
        let command: Box<dyn DbOperation + Send> = Box::from(command);
        multi_sender.send((tx.into(), command));

        //now wait
        match rc.await.unwrap() {
            ResponseFromDbOp::SweeperData(hashmap) => {
                hashmap.iter().for_each(|(field, _)| {
                //delete the key
                //form the delete command
                let command = Box::new(RemoveKey{key: field.clone()});
                multi_sender.send((None, command));
            });
            },
            _ => {},
        }
            
    }
}



pub async fn shutdown_listener(token: CancellationToken) {
    signal::ctrl_c()
        .await
        .expect("error registering interest in ctrl + c, cannot setup graceful shutdown");
    //once ctrl + c called, send a cancellation signal
    println!("ctrl + c called!");
    token.cancel();
}
