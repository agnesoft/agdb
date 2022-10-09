use agdb_multi_map::MultiMap;

#[test]
fn reserve_larger() {
    let mut map = MultiMap::<i64, i64>::new();
    map.insert(1, 1).unwrap();

    let capacity = map.capacity() + 10;
    let size = map.count();

    map.reserve(capacity).unwrap();

    assert_eq!(map.capacity(), capacity);
    assert_eq!(map.count(), size);
    assert_eq!(map.value(&1), Ok(Some(1)));
}

#[test]
fn reserve_same() {
    let mut map = MultiMap::<i64, i64>::new();
    map.insert(1, 1).unwrap();

    let capacity = map.capacity();
    let size = map.count();

    map.reserve(capacity).unwrap();

    assert_eq!(map.capacity(), capacity);
    assert_eq!(map.count(), size);
}

#[test]
fn reserve_smaller() {
    let mut map = MultiMap::<i64, i64>::new();
    map.insert(1, 1).unwrap();

    let current_capacity = map.capacity();
    let capacity = current_capacity - 10;
    let size = map.count();

    map.reserve(capacity).unwrap();

    assert_eq!(map.capacity(), current_capacity);
    assert_eq!(map.count(), size);
}
