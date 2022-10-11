use agdb_graph::GraphIndex;
use std::cmp::Ordering;

#[test]
fn derived_from_debug() {
    let index = GraphIndex::default();

    format!("{:?}", index);
}

#[test]
fn derived_from_ord() {
    let index = GraphIndex::default();
    assert_eq!(index.cmp(&index), Ordering::Equal);
}

#[test]
fn is_edge() {
    assert!(!GraphIndex::from(1).is_edge());
    assert!(!GraphIndex::default().is_edge());
    assert!(GraphIndex::from(-1).is_edge());
}

#[test]
fn is_node() {
    assert!(GraphIndex::from(1).is_node());
    assert!(!GraphIndex::default().is_node());
    assert!(!GraphIndex::from(-1).is_node());
}

#[test]
fn ordering() {
    let mut indexes = vec![
        GraphIndex::default(),
        GraphIndex::from(100_i64),
        GraphIndex::from(-1_i64),
        GraphIndex::from(1_i64),
    ];

    indexes.sort();

    assert_eq!(
        indexes,
        vec![
            GraphIndex::from(-1_i64),
            GraphIndex::default(),
            GraphIndex::from(1_i64),
            GraphIndex::from(100_i64),
        ]
    )
}
