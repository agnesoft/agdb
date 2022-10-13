use crate::search_control::SearchControl;
use crate::search_handler::SearchHandler;
use agdb_bit_set::BitSet;
use agdb_graph::Graph;
use agdb_graph::GraphIndex;

pub struct GraphSearch<'a> {
    pub(crate) graph: &'a Graph,
}

impl<'a> GraphSearch<'a> {
    pub fn breadth_first_search<T: SearchHandler>(
        &self,
        index: &GraphIndex,
        handler: &T,
    ) -> Vec<GraphIndex> {
        let mut distance = 0_u64;
        let mut result = Vec::<GraphIndex>::new();
        let mut stack = vec![index.clone()];
        let mut visited = BitSet::new();

        while !stack.is_empty() {
            distance += 1;
            let current = stack;
            stack = vec![];

            for i in current {
                if !visited.value(i.as_u64()) {
                    visited.insert(i.as_u64());

                    match handler.process(&i, &distance) {
                        SearchControl::Continue => result.push(i.clone()),
                        SearchControl::Finish => {
                            stack.clear();
                            break;
                        }
                        SearchControl::Skip => (),
                        SearchControl::Stop => continue,
                    }

                    if let Some(node) = self.graph.node(&i) {
                        for edge in node.edge_from_iter() {
                            stack.push(edge.index());
                        }
                    } else if let Some(edge) = self.graph.edge(&i) {
                        stack.push(edge.to())
                    }
                }
            }
        }

        result
    }
}
