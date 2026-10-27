//commands in the public interface are prety simple:
//GET, SET, EXPIRE (which sets a TTL for a key)
//the problm: internally, we need to perform much more complex operations like evict a certain key from the store.
//the thing is, these commands cannot be represented by the standard Command trait, becuase this trait is for user-facing commands
//some user-facing commands might have direct corresponding commands for db_operations.
//but why are we even doing this ?
// 1. single owner architecture leads away from mutex-based-architecture.
// 2. easier to visualize and plan, and pretty common
// 3. much better data consistency
// so, if there is a single owner of all the data (including keys and eviction policies),
// that owner provides channels to communicate over
// those channels expect Box<dyn Command> right now, but above I discussed that this trait does not fulfill the need for complex db operations
// conclusion: this new trait DbOperation

//concern: atomicity of user-facing commands. What if Command A executes some db operations between the db operation of Command B.

use std::{
    collections::HashMap, f32::consts::E, fmt::format, hash::{BuildHasher, Hash}, iter::Filter, time::Duration,
};

use tokio::time::Instant;

use crate::{ResponseFromDbOp, db::{Database, Entry}, expiry::{self, Expiry}};

//FOLLOWING ARE THE COMMANDS THAT WILL BE SENT OVER CHANNEL TO THE SINGLE DB ACTOR.
//NO MUTEXES, NO LOCKS, NO ARCS
pub trait DbOperation {
    fn execute(&self, db: &mut Database) -> ResponseFromDbOp;
}

pub struct GetKey {
    pub key: String,
}
pub struct SetKey {
    pub key: String,
    pub value: String,
    pub ttl: Option<u64>
}
pub struct SetEvictionPolicy {
    pub key: String,
    pub ttl: u64,
}
pub struct RemoveKey {
    pub key: String,
}

pub struct RemoveExpiredKeys {}

impl DbOperation for RemoveExpiredKeys {
    fn execute(&self, db: &mut Database) -> ResponseFromDbOp {
        //scan all the expired keys and remove them
        let keys_to_remove = db.get_expired_keys(100);
        keys_to_remove.iter().for_each(|key|{
            let command = RemoveKey {key: key.clone()};
            command.execute(db);
        });
        return ResponseFromDbOp::NormalResponse(format!("cleared expired keys up to {}", keys_to_remove.len()))
    }
}

impl DbOperation for GetKey {
    fn execute(&self, db: &mut Database) -> ResponseFromDbOp {
        //this checks if key exists or expired
        let Some(entry) = db.get_key_if_active(self.key.clone()) else {
            return ResponseFromDbOp::Error("Value does not exist!".into());
        };
        ResponseFromDbOp::NormalResponse(entry.0)
    }
}

impl DbOperation for SetKey {
    fn execute(&self, db: &mut Database) -> ResponseFromDbOp {
        //no expiry when 
        let mut expiry = None;
        if let Some(ttl) = self.ttl {
            expiry = Some(Expiry { ttl, now: Instant::now() })
        }
        ResponseFromDbOp::NormalResponse(match db.data.insert(self.key.clone(), (self.value.clone(), expiry)) {
            Some(_) => "Value update".to_string(),
            None => "Field did not exist, but created it".to_string(),
        })
    }
}

impl DbOperation for SetEvictionPolicy {
    fn execute(&self, db: &mut Database) -> ResponseFromDbOp {
        let expiry = Expiry {
            ttl: self.ttl,
            now: Instant::now(),
        };
        //check if key expired/removed
        if db.get_key_if_active(self.key.clone()).is_none() {
            return ResponseFromDbOp::Error(String::from("could not add TTL: key does not exist!"));
        };
        if db.set_expiry(self.key.clone(), expiry) {
            ResponseFromDbOp::NormalResponse(String::from("TTL added"))
        } else {
            ResponseFromDbOp::NormalResponse(String::from("TTL updated"))
        }
    }
}

impl DbOperation for RemoveKey {
    fn execute(&self, db: &mut Database) -> ResponseFromDbOp {
        let did_not_exist = db.data.remove(&self.key).is_none();
        //remove the entry from the sweeper context now
        // db.remove_expiry(self.key.clone());
        if did_not_exist {
            ResponseFromDbOp::NormalResponse(String::from("the key did not exist"))
        } else {
            ResponseFromDbOp::NormalResponse(String::from("key removed succesfully"))
        }
    }
}

// pub struct FilterKeysInData {
//     predicate: Box<dyn Fn((&String, &Entry)) -> bool + Send>,
//     limit: Option<usize>
// }


// impl DbOperation for FilterKeysInData {
//     //to be used later
//     fn execute(&self, db: &mut Database) -> ResponseFromDbOp {
//         ResponseFromDbOp::Data(
//             db.data
//             .iter()
//             .take(self.limit.unwrap_or(usize::MAX))
//             .filter(|(k, v)|(self.predicate)((k, v)))
//             .map(|(key, value)| (key.clone(), value.clone()))
//             .collect()
//         )
//     }
// }

//a command might call some db_operations
