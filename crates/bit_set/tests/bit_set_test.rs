use agdb_bit_set::BitSet;

#[test]
fn derived_from_default() {
    let _bitset = BitSet::default();
}

#[test]
fn insert() {
    let mut bitset = BitSet::new();

    assert!(!bitset.value(10_u64));

    bitset.insert(10_u64);

    assert!(bitset.value(10_u64));
}

#[test]
fn insert_multiple() {
    let mut bitset = BitSet::new();

    assert!(!bitset.value(10_u64));
    assert!(!bitset.value(11_u64));
    assert!(!bitset.value(2_u64));

    bitset.insert(10_u64);
    bitset.insert(11_u64);
    bitset.insert(2_u64);

    assert!(bitset.value(10_u64));
    assert!(bitset.value(11_u64));
    assert!(bitset.value(2_u64));
}

#[test]
fn remove() {
    let mut bitset = BitSet::new();

    bitset.insert(10_u64);
    bitset.insert(11_u64);
    bitset.insert(2_u64);

    bitset.remove(11_u64);

    assert!(bitset.value(10_u64));
    assert!(!bitset.value(11_u64));
    assert!(bitset.value(2_u64));
}

#[test]
fn remove_unset() {
    let mut bitset = BitSet::new();

    bitset.insert(10_u64);
    bitset.insert(11_u64);
    bitset.insert(2_u64);

    bitset.remove(9_u64);

    assert!(bitset.value(10_u64));
    assert!(bitset.value(11_u64));
    assert!(bitset.value(2_u64));
}

#[test]
fn remove_beyond_length() {
    let mut bitset = BitSet::new();

    bitset.insert(10_u64);
    bitset.insert(11_u64);
    bitset.insert(2_u64);

    bitset.remove(150_u64);

    assert!(bitset.value(10_u64));
    assert!(bitset.value(11_u64));
    assert!(bitset.value(2_u64));
}

#[test]
fn value_missing() {
    let mut bitset = BitSet::new();

    bitset.insert(5_u64);

    assert!(!bitset.value(2_u64));
}

#[test]
fn value_beyond_length() {
    let bitset = BitSet::new();

    assert!(!bitset.value(10_u64));
}
