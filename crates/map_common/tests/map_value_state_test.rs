use agdb_db_error::DbError;
use agdb_map_common::MapValueState;
use agdb_serialize::Serialize;

#[test]
fn bad_deserialization() {
    assert_eq!(
        MapValueState::deserialize(&[10_u8]),
        Err(DbError::from("value out of bounds"))
    );
}

#[test]
fn derived_from_default() {
    assert_eq!(MapValueState::default(), MapValueState::Empty);
}

#[test]
fn derived_from_debug() {
    let value = MapValueState::Deleted;

    format!("{:?}", value);
}

#[test]
fn serialize() {
    let data = vec![
        MapValueState::Valid,
        MapValueState::Empty,
        MapValueState::Deleted,
    ];
    let bytes = data.serialize();
    let other = Vec::<MapValueState>::deserialize(&bytes).unwrap();

    assert_eq!(data, other);
}

#[test]
fn serialized_size() {
    assert_eq!(MapValueState::serialized_size(), 1);
}
