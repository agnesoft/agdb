use crate::search_control::SearchControl;
use crate::search_handler::SearchHandler;
use agdb_bit_set::BitSet;
use agdb_graph::Graph;
use agdb_graph::GraphIndex;

pub struct GraphSearch<'a> {
    pub(crate) graph: &'a Graph,
    pub(crate) result: Vec<GraphIndex>,
    pub(crate) stack: Vec<GraphIndex>,
    pub(crate) visited: BitSet,
}

impl<'a> GraphSearch<'a> {
    pub fn breadth_first_search<T: SearchHandler>(
        &self,
        index: &GraphIndex,
        handler: &T,
    ) -> Vec<GraphIndex> {
        self.clear();
        let mut distance = 0_u64;

        while !self.stack.is_empty() {
            distance += 1;
            let current = self.stack;
            self.stack = vec![];

            for i in current {
                if !self.visited.value(i.as_u64()) {
                    self.visited.insert(i.as_u64());

                    match handler.process(&i, &distance) {
                        SearchControl::Continue => self.result.push(i.clone()),
                        SearchControl::Finish => {
                            self.stack.clear();
                            break;
                        }
                        SearchControl::Skip => (),
                        SearchControl::Stop => continue,
                    }

                    if let Some(node) = self.graph.node(&i) {
                        for edge in node.edge_from_iter() {
                            self.stack.push(edge.index());
                        }
                    } else if let Some(edge) = self.graph.edge(&i) {
                        self.stack.push(edge.to())
                    }
                }
            }
        }

        self.result
    }

    fn clear(&mut self) {
        self.result.clear();
        self.stack.clear();
        self.visited.clear();
    }
}
