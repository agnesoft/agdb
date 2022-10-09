use agdb_map_common::MapValue;
use agdb_map_common::MapValueState;
use agdb_serialize::Serialize;

#[test]
fn derived_from_debug() {
    let key_value = MapValue::<i64, i64>::default();

    format!("{:?}", key_value);
}

#[test]
fn derived_from_default() {
    let key_value = MapValue::<i64, i64>::default();

    assert_eq!(
        key_value,
        MapValue::<i64, i64> {
            state: MapValueState::Empty,
            key: 0,
            value: 0,
        }
    )
}

#[test]
fn i64_i64() {
    let key_value = MapValue {
        state: MapValueState::Valid,
        key: 1_i64,
        value: 10_i64,
    };
    let bytes = key_value.serialize();
    let other = MapValue::deserialize(&bytes);

    assert_eq!(other, Ok(key_value));
}

#[test]
fn out_of_bounds() {
    let bytes = vec![0_u8; 16];

    assert_eq!(
        MapValue::<i64, i64>::deserialize(&bytes)
            .unwrap_err()
            .description,
        "i64 deserialization error: out of bounds"
    );
}

#[test]
fn serialized_size() {
    assert_eq!(MapValue::<i64, i64>::serialized_size(), 17);
}
