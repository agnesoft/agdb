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
