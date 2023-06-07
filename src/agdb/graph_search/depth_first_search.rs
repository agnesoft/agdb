use super::search_impl::SearchIndex;
use super::search_impl::SearchIterator;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;

pub(crate) struct DepthFirstSearch {
    index: Option<SearchIndex>,
}

impl SearchIterator for DepthFirstSearch {
    fn expand_edge<Data: GraphData>(index: GraphIndex, graph: &GraphImpl<Data>) -> GraphIndex {
        graph
            .edge(index)
            .expect("invalid index, expected a valid edge index")
            .index_to()
    }

    fn expand_node<Data: GraphData>(index: GraphIndex, graph: &GraphImpl<Data>) -> Vec<GraphIndex> {
        graph
            .node(index)
            .expect("invalid index, expected a valid node index")
            .edge_iter_from()
            .map(|edge| edge.index())
            .collect()
    }

    fn new(stack: &mut Vec<SearchIndex>) -> Self {
        Self { index: stack.pop() }
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.index.take()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SearchControl;
    use super::super::SearchHandler;
    use super::*;
    use crate::graph::DbGraph;
    use crate::graph_search::GraphSearch;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct Handler {
        pub processor: fn(GraphIndex, u64) -> SearchControl,
    }

    impl Default for Handler {
        fn default() -> Self {
            Self {
                processor: |_index: GraphIndex, _distance: u64| SearchControl::Continue(true),
            }
        }
    }

    impl SearchHandler for Handler {
        fn process(&mut self, index: GraphIndex, distance: u64) -> SearchControl {
            (self.processor)(index, distance)
        }
    }

    #[test]
    fn empty_graph() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let graph = DbGraph::new(storage).unwrap();

        let result =
            GraphSearch::from(&graph).depth_first_search(GraphIndex::default(), Handler::default());

        assert_eq!(result, vec![]);
    }

    #[test]
    fn cyclic_graph() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node1, node2).unwrap();
        let edge3 = graph.insert_edge(node2, node3).unwrap();
        let edge4 = graph.insert_edge(node2, node3).unwrap();
        let edge5 = graph.insert_edge(node3, node1).unwrap();
        let edge6 = graph.insert_edge(node3, node1).unwrap();

        let result = GraphSearch::from(&graph).depth_first_search(node1, Handler::default());

        assert_eq!(
            result,
            vec![node1, edge1, node2, edge3, node3, edge5, edge6, edge4, edge2]
        );
    }

    #[test]
    fn full_search() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();
        let node4 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node1, node3).unwrap();
        let edge3 = graph.insert_edge(node1, node4).unwrap();

        let result = GraphSearch::from(&graph).depth_first_search(node1, Handler::default());

        assert_eq!(
            result,
            vec![node1, edge1, node2, edge2, node3, edge3, node4]
        );
    }

    #[test]
    fn filter_edges() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();
        let node4 = graph.insert_node().unwrap();

        graph.insert_edge(node1, node2).unwrap();
        graph.insert_edge(node1, node3).unwrap();
        graph.insert_edge(node1, node4).unwrap();

        let result = GraphSearch::from(&graph).depth_first_search(
            node1,
            Handler {
                processor: |index: GraphIndex, _distance: u64| {
                    SearchControl::Continue(index.is_node())
                },
            },
        );

        assert_eq!(result, vec![node1, node2, node3, node4]);
    }

    #[test]
    fn finish_search() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        graph.insert_edge(node1, node2).unwrap();
        graph.insert_edge(node1, node2).unwrap();
        graph.insert_edge(node2, node3).unwrap();
        graph.insert_edge(node2, node3).unwrap();
        graph.insert_edge(node3, node1).unwrap();
        graph.insert_edge(node3, node1).unwrap();

        let result = GraphSearch::from(&graph).depth_first_search(
            node1,
            Handler {
                processor: |index: GraphIndex, _distance: u64| {
                    if index.0 == 2 {
                        SearchControl::Finish(true)
                    } else {
                        SearchControl::Continue(false)
                    }
                },
            },
        );

        assert_eq!(result, vec![node2]);
    }

    #[test]
    fn search_twice() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();
        let node4 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node1, node3).unwrap();
        let edge3 = graph.insert_edge(node1, node4).unwrap();

        let mut result = GraphSearch::from(&graph).depth_first_search(node1, Handler::default());
        let expected = vec![node1, edge1, node2, edge2, node3, edge3, node4];

        assert_eq!(result, expected);

        result = GraphSearch::from(&graph).depth_first_search(node1, Handler::default());

        assert_eq!(result, expected);
    }

    #[test]
    fn stop_at_distance() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node1, node2).unwrap();
        let _edge3 = graph.insert_edge(node2, node3).unwrap();
        let _edge4 = graph.insert_edge(node3, node1).unwrap();

        let result = GraphSearch::from(&graph).depth_first_search(
            node1,
            Handler {
                processor: |_index: GraphIndex, distance: u64| {
                    if distance == 2 {
                        SearchControl::Stop(true)
                    } else {
                        SearchControl::Continue(true)
                    }
                },
            },
        );

        assert_eq!(result, vec![node1, edge1, node2, edge2]);
    }
}
