use agdb_dictionary::Dictionary;

#[test]
fn count_invalid_index() {
    let dictionary = Dictionary::<i64>::new();

    assert_eq!(dictionary.count(-1), Ok(None));
}

#[test]
fn default() {
    let _dictionary = Dictionary::<i64>::default();
}
