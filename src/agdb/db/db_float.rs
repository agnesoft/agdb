use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::cmp::Ordering;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug)]
pub struct DbFloat(f64);

impl DbFloat {
    pub fn to_f64(&self) -> f64 {
        self.0
    }
}

impl Eq for DbFloat {}

impl Hash for DbFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl Ord for DbFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for DbFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}

impl PartialOrd for DbFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.total_cmp(&other.0))
    }
}

impl From<f32> for DbFloat {
    fn from(value: f32) -> Self {
        DbFloat(value.into())
    }
}

impl From<f64> for DbFloat {
    fn from(value: f64) -> Self {
        DbFloat(value)
    }
}

impl Serialize for DbFloat {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(DbFloat::from(f64::deserialize(bytes)?))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }
}

impl StableHash for DbFloat {
    fn stable_hash(&self) -> u64 {
        self.0.to_bits().stable_hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    use std::collections::HashSet;

    #[test]
    fn derived_from_clone() {
        let float = DbFloat::from(1.0_f64);
        let _other = float.clone();
        let _other2 = float;
    }

    #[test]
    fn derived_from_debug() {
        format!("{:?}", DbFloat::from(1.0_f64));
    }

    #[test]
    fn eq() {
        let float = DbFloat::from(1.0_f64);
        let other = DbFloat::from(1.0_f64);

        assert_eq!(float, other);
    }

    #[test]
    fn from() {
        let _from_f32 = DbFloat::from(1.0_f32);
        let _from_f64 = DbFloat::from(1.0_f64);
    }

    #[test]
    fn hash() {
        let mut set = HashSet::<DbFloat>::new();
        set.insert(DbFloat::from(1.0_f64));
    }

    #[test]
    fn ord() {
        assert_eq!(DbFloat::from(1.0).cmp(&DbFloat::from(1.0)), Ordering::Equal);
    }

    #[test]
    fn partial_ord() {
        let mut vec = vec![
            DbFloat::from(1.1_f64),
            DbFloat::from(1.3_f64),
            DbFloat::from(-3.333_f64),
        ];
        vec.sort();

        assert_eq!(
            vec,
            vec![
                DbFloat::from(-3.333_f64),
                DbFloat::from(1.1_f64),
                DbFloat::from(1.3_f64)
            ]
        );
    }

    #[test]
    fn serialize() {
        let float = DbFloat::from(0.1_f64 + 0.2_f64);
        let bytes = float.serialize();
        let actual = DbFloat::deserialize(&bytes).unwrap();

        assert_eq!(float, actual);
    }

    #[test]
    fn stable_hash() {
        let hash = DbFloat::from(1.0_f64).stable_hash();

        assert_ne!(hash, 0);
    }

    #[test]
    fn to_f64() {
        let _to_f64 = DbFloat::from(1.0_f64).to_f64();
    }
}
