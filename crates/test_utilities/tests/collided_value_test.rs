use agdb_serialize::Serialize;
use agdb_test_utilities::CollidedValue;
use agdb_utilities::StableHash;

#[test]
fn derived_from_clone() {
    let value = CollidedValue::new(1_i64);
    let other = value.clone();

    assert_eq!(value, other);
}

#[test]
fn derived_from_debug() {
    let value = CollidedValue::new(1_i64);

    format!("{:?}", value);
}

#[test]
fn serialize() {
    let value = CollidedValue::new(1_i64);
    let bytes = value.serialize();
    let other = CollidedValue::deserialize(&bytes).unwrap();

    assert_eq!(value, other);
}

#[test]
fn stable_hash() {
    let value = CollidedValue::new(1_i64);

    assert_eq!(value.stable_hash(), 1_u64);
}
