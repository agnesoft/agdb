use crate::collided_value::CollidedValue;
use agdb_utilities::StableHash;

impl<T> StableHash for CollidedValue<T> {
    fn stable_hash(&self) -> u64 {
        1
    }
}
