use super::path::Path;
use super::path_search_handler::PathSearchHandler;
use crate::collections::bit_set::BitSet;
use crate::graph::graph_data::GraphData;
use crate::graph::graph_impl::GraphImpl;
use crate::graph::graph_index::GraphIndex;
use std::cmp::Ordering;
use std::mem::swap;
use std::mem::take;

pub(crate) struct PathSearch<'a, Data, Handler>
where
    Data: GraphData,
    Handler: PathSearchHandler,
{
    pub(crate) current_path: Path,
    pub(crate) destination: GraphIndex,
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) handler: &'a Handler,
    pub(crate) paths: Vec<Path>,
    pub(crate) result: Vec<GraphIndex>,
    pub(crate) visited: BitSet,
}

impl<'a, Data, Handler> PathSearch<'a, Data, Handler>
where
    Data: GraphData,
    Handler: PathSearchHandler,
{
    pub(crate) fn new(
        graph: &'a GraphImpl<Data>,
        from: GraphIndex,
        to: GraphIndex,
        handler: &'a Handler,
    ) -> Self {
        Self {
            current_path: Path {
                elements: vec![],
                cost: 0,
            },
            destination: to,
            graph,
            handler,
            paths: vec![Path {
                elements: vec![from],
                cost: 0,
            }],
            result: vec![],
            visited: BitSet::new(),
        }
    }

    pub(crate) fn search(&mut self) -> Vec<GraphIndex> {
        while !self.is_finished() {
            self.sort_paths();
            self.process_last_path();
        }

        take(&mut self.result)
    }

    fn expand_edge(&mut self, mut path: Path, index: &GraphIndex, node_index: &GraphIndex) {
        let cost = self
            .handler
            .process(index, &(self.current_path.elements.len() as u64 + 1));

        if cost != 0 && !self.visited.value(node_index.as_u64()) {
            path.elements.push(index.clone());
            path.cost += cost;
            self.expand_node(path, node_index);
        }
    }

    fn expand_node(&mut self, mut path: Path, index: &GraphIndex) {
        let cost = self
            .handler
            .process(index, &(self.current_path.elements.len() as u64 + 1));

        if cost != 0 {
            path.elements.push(index.clone());
            path.cost += cost;
            self.paths.push(path);
        }
    }

    fn expand(&mut self, index: &GraphIndex) {
        let node = self
            .graph
            .node(index)
            .expect("unexpected invalid node index");
        for edge in node.edge_iter_from() {
            self.expand_edge(self.current_path.clone(), &edge.index(), &edge.index_to());
        }
    }

    fn is_finished(&self) -> bool {
        self.paths.is_empty() || !self.result.is_empty()
    }

    fn process_path(&mut self) {
        let index = self
            .current_path
            .elements
            .last()
            .map_or(GraphIndex::default(), |index| index.clone());
        self.process_index(&index);
    }

    fn process_index(&mut self, index: &GraphIndex) {
        if !self.visited.value(index.as_u64()) {
            if index.value() == self.destination.value() {
                swap(&mut self.result, &mut self.current_path.elements);
            } else {
                self.visited.insert(index.as_u64());
                self.expand(index);
            }
        }
    }

    fn process_last_path(&mut self) {
        self.current_path = self.paths.pop().unwrap_or(Path {
            elements: vec![],
            cost: 0,
        });
        self.process_path();
    }

    fn sort_paths(&mut self) {
        self.paths.sort_by(|left, right| {
            let ordering = left.cost.cmp(&right.cost);

            if ordering == Ordering::Equal {
                return left.elements.len().cmp(&right.elements.len()).reverse();
            }

            ordering.reverse()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::graph_search::GraphSearch;

    struct Handler {
        pub processor: fn(&GraphIndex, &u64) -> u64,
    }

    impl Default for Handler {
        fn default() -> Self {
            Self {
                processor: |_index: &GraphIndex, _distance: &u64| 1_u64,
            }
        }
    }

    impl PathSearchHandler for Handler {
        fn process(&self, index: &GraphIndex, distance: &u64) -> u64 {
            (self.processor)(index, distance)
        }
    }

    #[test]
    fn circular_path() {
        let mut graph = Graph::new();
        let node = graph.insert_node().unwrap();
        let _edge = graph.insert_edge(&node, &node).unwrap();

        let result = GraphSearch::from(&graph).path(&node, &node, &Handler::default());

        assert_eq!(result, vec![]);
    }

    #[test]
    fn empty_graph() {
        let graph = Graph::new();

        let result = GraphSearch::from(&graph).path(
            &GraphIndex::default(),
            &GraphIndex::default(),
            &Handler::default(),
        );

        assert_eq!(result, vec![]);
    }

    #[test]
    fn multi_edge_path() {
        let mut graph = Graph::new();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(&node1, &node2).unwrap();
        let _edge2 = graph.insert_edge(&node1, &node2).unwrap();

        let edge3 = graph.insert_edge(&node2, &node3).unwrap();
        let _edge4 = graph.insert_edge(&node2, &node3).unwrap();

        let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

        assert_eq!(result, vec![node1, edge1, node2, edge3, node3]);
    }

    #[test]
    fn origin_is_destination() {
        let mut graph = Graph::new();
        let node = graph.insert_node().unwrap();

        let result = GraphSearch::from(&graph).path(&node, &node, &Handler::default());

        assert_eq!(result, vec![]);
    }

    #[test]
    fn short_circuit_path() {
        let mut graph = Graph::new();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(&node1, &node3).unwrap();
        let _edge2 = graph.insert_edge(&node1, &node2).unwrap();
        let _edge3 = graph.insert_edge(&node2, &node3).unwrap();

        let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

        assert_eq!(result, vec![node1, edge1, node3]);
    }

    #[test]
    fn single_path() {
        let mut graph = Graph::new();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(&node1, &node2).unwrap();
        let edge2 = graph.insert_edge(&node2, &node3).unwrap();

        let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

        assert_eq!(result, vec![node1, edge1, node2, edge2, node3]);
    }

    #[test]
    fn skip_short_circuit_path() {
        let mut graph = Graph::new();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let _edge1 = graph.insert_edge(&node1, &node3).unwrap();
        let edge2 = graph.insert_edge(&node1, &node2).unwrap();
        let edge3 = graph.insert_edge(&node2, &node3).unwrap();

        let result = GraphSearch::from(&graph).path(
            &node1,
            &node3,
            &Handler {
                processor: |index: &GraphIndex, _distance: &u64| {
                    if index.value() == -4 {
                        return 0;
                    }

                    1
                },
            },
        );

        assert_eq!(result, vec![node1, edge2, node2, edge3, node3]);
    }

    #[test]
    fn unconnected() {
        let mut graph = Graph::new();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let _edge1 = graph.insert_edge(&node1, &node2).unwrap();

        let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

        assert_eq!(result, vec![]);
    }
}
