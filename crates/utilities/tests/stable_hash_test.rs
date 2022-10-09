use agdb_utilities::StableHash;

#[test]
fn i64() {
    assert_eq!(10_i64.stable_hash(), 10_u64);
}

#[test]
fn u64() {
    assert_eq!(10_u64.stable_hash(), 10_u64);
}
