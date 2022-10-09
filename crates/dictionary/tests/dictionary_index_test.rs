use agdb_dictionary::Dictionary;
use agdb_test_utilities::CollidedValue;

#[test]
fn index() {
    let mut dictionary = Dictionary::<i64>::new();

    let index = dictionary.insert(&10).unwrap();

    assert_eq!(dictionary.index(&10), Ok(Some(index)));
}

#[test]
fn index_missing_value() {
    let dictionary = Dictionary::<i64>::new();

    assert_eq!(dictionary.index(&10), Ok(None));
}

#[test]
fn index_removed_value() {
    let mut dictionary = Dictionary::<i64>::new();

    let index = dictionary.insert(&10).unwrap();
    dictionary.remove(index).unwrap();

    assert_eq!(dictionary.index(&10), Ok(None));
}

#[test]
fn index_reuse() {
    let mut dictionary = Dictionary::<i64>::new();

    let index1 = dictionary.insert(&5).unwrap();
    let index2 = dictionary.insert(&10).unwrap();
    let index3 = dictionary.insert(&7).unwrap();

    dictionary.remove(index2).unwrap();
    dictionary.remove(index1).unwrap();
    dictionary.remove(index3).unwrap();

    assert_eq!(dictionary.count(index1), Ok(None));
    assert_eq!(dictionary.count(index2), Ok(None));
    assert_eq!(dictionary.count(index3), Ok(None));

    assert_eq!(dictionary.insert(&3), Ok(index3));
    assert_eq!(dictionary.insert(&2), Ok(index1));
    assert_eq!(dictionary.insert(&1), Ok(index2));

    assert_eq!(dictionary.value(index1), Ok(Some(2)));
    assert_eq!(dictionary.value(index2), Ok(Some(1)));
    assert_eq!(dictionary.value(index3), Ok(Some(3)));
}

#[test]
fn index_with_collisions() {
    let mut dictionary = Dictionary::<CollidedValue<i64>>::new();

    let index1 = dictionary.insert(&CollidedValue::new(1)).unwrap();
    let index2 = dictionary.insert(&CollidedValue::new(2)).unwrap();
    let index3 = dictionary.insert(&CollidedValue::new(3)).unwrap();

    assert_eq!(dictionary.index(&CollidedValue::new(1)), Ok(Some(index1)));
    assert_eq!(dictionary.index(&CollidedValue::new(2)), Ok(Some(index2)));
    assert_eq!(dictionary.index(&CollidedValue::new(3)), Ok(Some(index3)));
}

#[test]
fn value_missing_index() {
    let dictionary = Dictionary::<i64>::new();

    assert_eq!(dictionary.value(1), Ok(None));
}
