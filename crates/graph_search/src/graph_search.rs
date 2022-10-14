use std::mem::swap;

use crate::graph_search_index::GraphSearchIndex;
use crate::search_control::SearchControl;
use crate::search_handler::SearchHandler;
use agdb_bit_set::BitSet;
use agdb_graph::Graph;
use agdb_graph::GraphIndex;

pub struct GraphSearch<'a> {
    pub(crate) graph: &'a Graph,
    pub(crate) stack: Vec<GraphSearchIndex>,
    pub(crate) visited: BitSet,
    pub(crate) result: Vec<GraphIndex>,
}

impl<'a> GraphSearch<'a> {
    pub fn breadth_first_search<T: SearchHandler>(
        &mut self,
        index: &GraphIndex,
        handler: &T,
    ) -> Vec<GraphIndex> {
        self.clear();

        self.stack.push(GraphSearchIndex {
            index: index.clone(),
            distance: 0,
        });

        while !self.stack.is_empty() {
            for i in self.take_stack() {
                if !self.visit(&i) {
                    match handler.process(&i.index, &i.distance) {
                        SearchControl::Continue => self.result.push(i.index.clone()),
                        SearchControl::Finish => {
                            self.stack.clear();
                            break;
                        }
                        SearchControl::Skip => (),
                        SearchControl::Stop => continue,
                    }

                    if let Some(node) = self.graph.node(&i.index) {
                        for edge in node.edge_from_iter() {
                            self.stack.push(GraphSearchIndex {
                                index: edge.index(),
                                distance: i.distance + 1,
                            });
                        }
                    } else if let Some(edge) = self.graph.edge(&i.index) {
                        self.stack.push(GraphSearchIndex {
                            index: edge.to(),
                            distance: i.distance + 1,
                        })
                    }
                }
            }
        }

        let mut res = Vec::<GraphIndex>::new();
        swap(&mut res, &mut self.result);
        res
    }

    fn clear(&mut self) {
        self.result.clear();
        self.stack.clear();
        self.visited.clear();
    }

    fn take_stack(&mut self) -> Vec<GraphSearchIndex> {
        let mut res = Vec::<GraphSearchIndex>::new();
        swap(&mut res, &mut self.stack);

        res
    }

    fn visit(&mut self, index: &GraphSearchIndex) -> bool {
        let visited = self.visited.value(index.index.as_u64());
        self.visited.insert(index.index.as_u64());

        visited
    }
}
