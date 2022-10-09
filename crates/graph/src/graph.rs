use crate::graph_data_memory::GraphDataMemory;
use crate::graph_impl::GraphImpl;

pub type Graph = GraphImpl<GraphDataMemory>;

impl Graph {
    pub fn new() -> Graph {
        Graph {
            data: GraphDataMemory::default(),
        }
    }
}
