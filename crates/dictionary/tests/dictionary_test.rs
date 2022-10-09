use agdb_dictionary::Dictionary;
use agdb_dictionary::DictionaryIndex;

#[test]
fn count_invalid_index() {
    let dictionary = Dictionary::<i64>::new();

    assert_eq!(dictionary.count(&DictionaryIndex::default()), Ok(None));
    assert_eq!(dictionary.count(&DictionaryIndex::from(-1_i64)), Ok(None));
}

#[test]
fn default() {
    let _dictionary = Dictionary::<i64>::default();
}

#[test]
fn value_missing_index() {
    let dictionary = Dictionary::<i64>::new();
    assert_eq!(dictionary.value(&DictionaryIndex::from(1_i64)), Ok(None));
}
