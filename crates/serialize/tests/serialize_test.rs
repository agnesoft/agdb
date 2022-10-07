use db_error::DbError;
use serialize::Serialize;

#[test]
fn i64() {
    let number = -10_i64;
    let bytes = number.serialize();
    let actual = i64::deserialize(&bytes);

    assert_eq!(actual, Ok(number));
}

#[test]
fn i64_out_of_bounds() {
    let bytes = vec![0_u8; 4];

    assert_eq!(
        i64::deserialize(&bytes),
        Err(DbError::from("i64 deserialization error: out of bounds"))
    );
}

#[test]
fn serialized_size() {
    assert_eq!(i64::serialized_size(), 8);
    assert_eq!(u64::serialized_size(), 8);
    assert_eq!(Vec::<i64>::serialized_size(), 0);
    assert_eq!(String::serialized_size(), 0);
}

#[test]
fn string() {
    let value = "Hello, World!".to_string();
    let bytes = value.serialize();
    let actual = String::deserialize(&bytes);

    assert_eq!(actual, Ok(value));
}

#[test]
fn string_bad_bytes() {
    let bad_bytes = vec![0xdf, 0xff];

    assert!(String::deserialize(&bad_bytes).is_err());
}

#[test]
fn u64() {
    let number = 10_u64;
    let bytes = number.serialize();
    let actual = u64::deserialize(&bytes);

    assert_eq!(actual, Ok(number));
}

#[test]
fn u64_out_of_bounds() {
    let bytes = vec![0_u8; 4];

    assert_eq!(
        u64::deserialize(&bytes),
        Err(DbError::from("u64 deserialization error: out of bounds"))
    );
}

#[test]
fn vec_i64() {
    let data = vec![1_i64, 2_i64, 3_i64];
    let bytes = data.serialize();
    let actual = Vec::<i64>::deserialize(&bytes);

    assert_eq!(actual, Ok(data));
}

#[test]
fn vec_size_out_of_bounds() {
    let bytes = vec![0_u8; 4];

    assert_eq!(
        Vec::<i64>::deserialize(&bytes),
        Err(DbError::from("u64 deserialization error: out of bounds"))
    );
}

#[test]
fn vec_u8() {
    let data = vec![1_u8, 2_u8, 3_u8];
    let bytes = data.serialize();
    let actual = Vec::<u8>::deserialize(&bytes);

    assert_eq!(actual, Ok(data));
}

#[test]
fn vec_value_out_of_bounds() {
    let bytes = 1_u64.serialize();

    assert_eq!(
        Vec::<i64>::deserialize(&bytes),
        Err(DbError::from("i64 deserialization error: out of bounds"))
    );
}
