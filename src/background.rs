use std::{sync::Arc, time::Duration};

use tokio::{spawn, time::{Instant, sleep}};
use zyncdb::Database;

pub async fn bg_sweeper(db: Arc<Database>) {
    //responsible for checking the expiry 
    //todo: use channels instead of periodically polling
    
   loop {
    sleep(Duration::from_secs(1)).await;
    //wake up after atleast 1 secs everytime
    //check all the expiries
    db.sweeper_context.lock().unwrap().iter().filter(|(_, expiry)| {
        (Instant::now() - expiry.now) >= Duration::from_secs(expiry.ttl)
    }).for_each(|( field, _)|{
        //spawn an async task, allow to extend in the future
        let db_to_pass = db.clone();
        spawn(field_delete(db_to_pass, field.clone()));
    });
   }
}

pub async fn field_delete(db: Arc<Database>, field: String) {
    db.data.lock().unwrap().remove(&field);
}