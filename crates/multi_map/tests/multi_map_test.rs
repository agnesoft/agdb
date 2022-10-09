use agdb_multi_map::MultiMap;

#[test]
fn derived_from_default() {
    let mut _map = MultiMap::<i64, i64>::default();
}

#[test]
fn iter() {
    let mut map = MultiMap::<i64, i64>::new();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();
    map.insert(2, 30).unwrap();
    map.insert(2, 50).unwrap();
    map.insert(4, 13).unwrap();
    map.remove_key(&7).unwrap();

    let mut actual = map.iter().collect::<Vec<(i64, i64)>>();
    actual.sort();
    let expected: Vec<(i64, i64)> = vec![(1, 10), (2, 30), (2, 50), (4, 13), (5, 15), (5, 15)];

    assert_eq!(actual, expected);
}

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
