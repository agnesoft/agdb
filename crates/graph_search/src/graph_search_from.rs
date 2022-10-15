use crate::graph_search::GraphSearch;
use agdb_bit_set::BitSet;
use agdb_graph::GraphData;
use agdb_graph::GraphImpl;

impl<'a, Data> From<&'a GraphImpl<Data>> for GraphSearch<'a, Data>
where
    Data: GraphData,
{
    fn from(graph: &'a GraphImpl<Data>) -> Self {
        GraphSearch {
            graph,
            visited: BitSet::new(),
            stack: vec![],
            result: vec![],
        }
    }
}
