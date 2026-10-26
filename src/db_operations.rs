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

use tokio::time::Instant;

use crate::{db::Database, expiry::Expiry};

//FOLLOWING ARE THE COMMANDS THAT WILL BE SENT OVER CHANNEL TO THE SINGLE DB ACTOR.
//NO MUTEXES, NO LOCKS, NO ARCS
pub trait DbOperation {
    fn execute(&self, db: &mut Database) -> String;
}

pub struct GetKey {
    key: String,
}
pub struct SetKey {
    key: String,
    value: String,
}
pub struct SetEvictionPolicy {
    key: String,
    ttl: u64,
}
pub struct RemoveKey {
    key: String,
}

impl DbOperation for GetKey {
    fn execute(&self, db: &mut Database) -> String {
        match db.data.get(&self.key) {
            Some(value) => value.to_owned(),
            None => "Value does not exist!".to_string(),
        }
    }
}

impl DbOperation for SetKey {
    fn execute(&self, db: &mut Database) -> String {
        match db.data.insert(self.key.clone(), self.value.clone()) {
            Some(_) => "Value update".to_string(),
            None => "Field did not exist, but created it".to_string(),
        }
    }
}

impl DbOperation for SetEvictionPolicy {
    fn execute(&self, db: &mut Database) -> String {
        let expiry = Expiry {
            ttl: self.ttl,
            now: Instant::now(),
        };
        if db.set_expiry(self.key.clone(), expiry) {
            String::from("TTL added")
        } else {
            String::from("TTL updated")
        }
    }
}

impl DbOperation for RemoveKey {
    fn execute(&self, db: &mut Database) -> String {
        let did_not_exist = db.data.remove(&self.key).is_none();
        //remove the entry from the sweeper context now
        db.remove_expiry(self.key.clone());
        if did_not_exist {
            String::from("the key did not exist")
        }else {
            String::from("key removed succesfully")
        }
    }
}

//a command might call some db_operations
