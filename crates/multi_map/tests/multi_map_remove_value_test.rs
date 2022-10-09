use agdb_multi_map::MultiMap;

#[test]
fn remove_value() {
    let mut map = MultiMap::<i64, i64>::new();

    map.insert(1, 7).unwrap();
    map.insert(5, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(5, 20).unwrap();

    assert_eq!(map.count(), 4);
    map.remove_value(&5, &15).unwrap();

    assert_eq!(map.count(), 3);
    assert_eq!(map.value(&1), Ok(Some(7)));
    assert_eq!(map.values(&5), Ok(vec![10_i64, 20_i64]));
}

#[test]
fn remove_missing_value() {
    let mut map = MultiMap::<i64, i64>::new();

    map.remove_value(&5, &10).unwrap();

    assert_eq!(map.count(), 0);
}
