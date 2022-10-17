use crate::breadth_first_search::BreadthFirstSearch;
use crate::depth_first_search::DepthFirstSearch;
use crate::search_handler::SearchHandler;
use crate::search_impl::SearchImpl;
use agdb_graph::GraphData;
use agdb_graph::GraphImpl;
use agdb_graph::GraphIndex;

pub struct GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
}

impl<'a, Data> GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub fn breadth_first_search<Handler: SearchHandler>(
        &mut self,
        index: &GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.validate_index(index) {
            SearchImpl::<'a, Data, BreadthFirstSearch>::new(self.graph, index.clone())
                .search(handler)
        } else {
            vec![]
        }
    }

    pub fn depth_first_search<Handler: SearchHandler>(
        &mut self,
        index: &GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.validate_index(index) {
            SearchImpl::<'a, Data, DepthFirstSearch>::new(self.graph, index.clone()).search(handler)
        } else {
            vec![]
        }
    }

    fn validate_index(&self, index: &GraphIndex) -> bool {
        self.graph.node(index).is_some() || self.graph.edge(index).is_some()
    }
}
