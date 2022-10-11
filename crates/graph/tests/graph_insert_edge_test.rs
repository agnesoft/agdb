use agdb_db_error::DbError;
use agdb_graph::Graph;
use agdb_graph::GraphIndex;

#[test]
fn insert_edge() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();

    assert_eq!(graph.insert_edge(&from, &to), Ok(GraphIndex::from(-3_i64)));
}

#[test]
fn insert_edge_after_removed() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index = graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index).unwrap();

    assert_eq!(graph.insert_edge(&from, &to), Ok(index));
}

#[test]
fn insert_edge_after_several_removed() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(&from, &to).unwrap();
    let index2 = graph.insert_edge(&from, &to).unwrap();
    graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index1).unwrap();
    graph.remove_edge(&index2).unwrap();

    assert_eq!(graph.insert_edge(&from, &to), Ok(index2));
}

#[test]
fn insert_edge_invalid_from() {
    let mut graph = Graph::new();

    assert_eq!(
        graph.insert_edge(&GraphIndex::from(1), &GraphIndex::from(2)),
        Err(DbError::from("'1' is invalid index"))
    );
}

#[test]
fn insert_edge_invalid_to() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();

    assert_eq!(
        graph.insert_edge(&from, &GraphIndex::from(2)),
        Err(DbError::from("'2' is invalid index"))
    );
}
