use agdb_multi_map::MultiMap;

#[test]
fn value_missing() {
    let map = MultiMap::<i64, i64>::new();

    assert_eq!(map.value(&0), Ok(None));
}

#[test]
fn values_at_end() {
    let mut map = MultiMap::<i64, i64>::new();

    map.insert(127, 10).unwrap();
    map.insert(255, 11).unwrap();
    map.insert(191, 12).unwrap();

    assert_eq!(map.value(&127), Ok(Some(10)));
    assert_eq!(map.value(&255), Ok(Some(11)));
    assert_eq!(map.value(&191), Ok(Some(12)));
}
