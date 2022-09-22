mod graph_edge;
mod graph_element;
mod graph_node;

pub(crate) struct Graph {
    from: Vec<i64>,
    to: Vec<i64>,
    from_meta: Vec<i64>,
    to_meta: Vec<i64>,
    node_count: u64,
}

#[allow(dead_code)]
impl Graph {
    pub(crate) fn new() -> Graph {
        Graph {
            from: vec![0],
            to: vec![0],
            from_meta: vec![0],
            to_meta: vec![0],
            node_count: 0,
        }
    }

    pub(crate) fn insert_node(&mut self) -> i64 {
        self.from.push(0);
        self.to.push(0);
        self.from_meta.push(0);
        self.to_meta.push(0);
        self.node_count += 1;
        self.node_count as i64
    }
}

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
