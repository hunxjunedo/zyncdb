use std::{time::Duration};
use tokio::{
    signal, sync::{mpsc, oneshot}, time::sleep,
};
use tokio_util::sync::CancellationToken;

use crate::{MainMpscChannelMessage, ResponseFromDbOp, db_operations::{DbOperation, RemoveExpiredKeys, RemoveKey}, expiry::Expiry, read_env_var};

pub async fn bg_sweeper(multi_sender: mpsc::Sender<MainMpscChannelMessage>) {
    //responsible for checking the expiry
    //todo: use channels instead of periodically polling
    //todo: fix the race condition this causes DONE
    //todo: update expiry when a new set occurs DONE

    loop {
        sleep(Duration::from_secs(read_env_var("SWEEPER_FREQUENCY"))).await;
        let (tx, rc) = oneshot::channel::<ResponseFromDbOp>();
        _ = multi_sender.send((tx.into(), Box::new(RemoveExpiredKeys{}))).await;
        let response = rc.await.unwrap();
        if let ResponseFromDbOp::NormalResponse(resp) = response {
            println!("{}", resp);
        };
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
