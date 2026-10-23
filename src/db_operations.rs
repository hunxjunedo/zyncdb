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