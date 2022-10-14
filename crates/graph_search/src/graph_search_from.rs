use crate::graph_search::GraphSearch;
use agdb_bit_set::BitSet;
use agdb_graph::Graph;

impl<'a> From<&'a Graph> for GraphSearch<'a> {
    fn from(graph: &'a Graph) -> Self {
        GraphSearch {
            graph,
            visited: BitSet::new(),
            result: vec![],
        }
    }
}
