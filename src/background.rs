use std::{sync::Arc, time::Duration};

use tokio::{
    signal, spawn,
    time::{Instant, sleep},
};
use tokio_util::sync::CancellationToken;

use crate::{db::Database, read_env_var};

pub async fn bg_sweeper(db: Arc<Database>) {
    //responsible for checking the expiry
    //todo: use channels instead of periodically polling

    loop {
        sleep(Duration::from_secs(read_env_var("SWEEPER_FREQUENCY"))).await;
        //wake up after atleast 1 secs everytime
        //check all the expiries
        db.sweeper_context
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, expiry)| expiry.now.elapsed() >= Duration::from_secs(expiry.ttl))
            .for_each(|(field, _)| {
                //spawn an async task, allow to extend in the future
                let db_to_pass = db.clone();
                spawn(field_delete(db_to_pass, field.clone()));
            });
    }
}

pub async fn field_delete(db: Arc<Database>, field: String) {
    db.data.lock().unwrap().remove(&field);
    //remove the entry from the sweeper context now
    db.remove_expiry(field);
}

pub async fn shutdown_listener(token: CancellationToken) {
    signal::ctrl_c()
        .await
        .expect("error registering interest in ctrl + c, cannot setup graceful shutdown");
    //once ctrl + c called, send a cancellation signal
    println!("ctrl + c called!");
    token.cancel();
}
