use crate::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::cmp::Ordering;
use std::hash::Hash;
use std::hash::Hasher;

/// Database float is a wrapper around `f64` to provide
/// functionality like comparison. The comparison is
/// using `total_cmp` standard library function. See its
/// [docs](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp)
/// to understand how it handles NaNs and other edge cases
/// of floating point numbers.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "api", derive(agdb::TypeDefImpl))]
pub struct DbF64(f64);

impl DbF64 {
    /// Extracts the underlying `f64` value.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_f64(&self) -> f64 {
        self.0
    }
}

impl Eq for DbF64 {}

impl Hash for DbF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl Ord for DbF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for DbF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}

impl PartialOrd for DbF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl From<f32> for DbF64 {
    fn from(value: f32) -> Self {
        DbF64(value.into())
    }
}

impl From<f64> for DbF64 {
    fn from(value: f64) -> Self {
        DbF64(value)
    }
}

impl Serialize for DbF64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(DbF64::from(f64::deserialize(bytes)?))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    fn serialized_size(&self) -> u64 {
        self.0.serialized_size()
    }
}

impl StableHash for DbF64 {
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
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let float = DbF64::from(1.0_f64);
        let _other = float.clone();
        let _other2 = float;
    }
    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", DbF64::from(1.0_f64));
    }

    #[test]
    fn derived_from_eq() {
        let float = DbF64::from(1.0_f64);
        let other = DbF64::from(1.0_f64);

        assert_eq!(float, other);
    }

    #[test]
    fn derived_from_hash() {
        let mut set = HashSet::<DbF64>::new();
        set.insert(DbF64::from(1.0_f64));
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DbF64::from(1.0).cmp(&DbF64::from(1.0)), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut vec = vec![
            DbF64::from(1.1_f64),
            DbF64::from(1.3_f64),
            DbF64::from(-3.333_f64),
        ];
        vec.sort();

        assert_eq!(
            vec,
            vec![
                DbF64::from(-3.333_f64),
                DbF64::from(1.1_f64),
                DbF64::from(1.3_f64)
            ]
        );
    }

    #[test]
    fn from_f32() {
        let _ = DbF64::from(1.0_f32);
    }

    #[test]
    fn from_f64() {
        let _ = DbF64::from(1.0_f64);
    }

    #[test]
    fn serialize() {
        let float = DbF64::from(0.1_f64 + 0.2_f64);
        let bytes = float.serialize();

        assert_eq!(bytes.len() as u64, float.serialized_size());

        let actual = DbF64::deserialize(&bytes).unwrap();

        assert_eq!(float, actual);
    }

    #[test]
    fn stable_hash() {
        let hash = DbF64::from(1.0_f64).stable_hash();

        assert_ne!(hash, 0);
    }

    #[test]
    fn to_f64() {
        let _to_f64 = DbF64::from(1.0_f64).to_f64();
    }
}
