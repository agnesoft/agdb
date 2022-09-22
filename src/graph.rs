mod graph_edge;
mod graph_element;
mod graph_node;

#[allow(dead_code)]
pub(crate) struct Graph {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_node() {
        let mut graph = Graph::new();
        let id = graph.insert_node();

        assert_eq!(id, 1);
    }
}
