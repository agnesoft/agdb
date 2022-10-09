use agdb_dictionary::DictionaryIndex;

#[test]
fn derived_from_debug() {
    let index = DictionaryIndex::default();

    format!("{:?}", index);
}
