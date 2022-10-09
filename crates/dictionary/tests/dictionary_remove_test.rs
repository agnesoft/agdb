use agdb_dictionary::Dictionary;
use agdb_dictionary::DictionaryIndex;

#[test]
fn remove() {
    let mut dictionary = Dictionary::<i64>::new();

    let index = dictionary.insert(&10).unwrap();
    dictionary.remove(&index).unwrap();

    assert_eq!(dictionary.value(&index), Ok(None));
    assert_eq!(dictionary.count(&index), Ok(None));
}

#[test]
fn remove_duplicated() {
    let mut dictionary = Dictionary::<i64>::new();

    let index = dictionary.insert(&10).unwrap();
    dictionary.insert(&10).unwrap();
    dictionary.insert(&10).unwrap();

    assert_eq!(dictionary.value(&index), Ok(Some(10)));
    assert_eq!(dictionary.count(&index), Ok(Some(3)));

    dictionary.remove(&index).unwrap();

    assert_eq!(dictionary.value(&index), Ok(Some(10)));
    assert_eq!(dictionary.count(&index), Ok(Some(2)));

    dictionary.remove(&index).unwrap();
    dictionary.remove(&index).unwrap();

    assert_eq!(dictionary.value(&index), Ok(None));
    assert_eq!(dictionary.count(&index), Ok(None));
}

#[test]
fn remove_missing() {
    let mut dictionary = Dictionary::<i64>::new();

    let index = dictionary.insert(&10).unwrap();

    assert_eq!(dictionary.len(), Ok(1));

    dictionary
        .remove(&DictionaryIndex::from(index.value() + 1))
        .unwrap();

    assert_eq!(dictionary.len(), Ok(1));
}
