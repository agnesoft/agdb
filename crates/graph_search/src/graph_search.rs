use crate::breadth_first_search::BreadthFirstSearch;
use crate::depth_first_search::DepthFirstSearch;
use crate::path_search_impl::PathSearchImpl;
use crate::search_handler::SearchHandler;
use crate::search_impl::SearchImpl;
use crate::PathSearchHandler;
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
        &self,
        index: &GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, BreadthFirstSearch>::new(self.graph, index.clone())
                .search(handler)
        } else {
            vec![]
        }
    }

    pub fn depth_first_search<Handler: SearchHandler>(
        &self,
        index: &GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, DepthFirstSearch>::new(self.graph, index.clone()).search(handler)
        } else {
            vec![]
        }
    }

    pub fn path<Handler: PathSearchHandler>(
        &self,
        from: &GraphIndex,
        to: &GraphIndex,
        handler: &'a Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_node(from) && self.is_valid_node(to) {
            PathSearchImpl::<'a, Data, Handler>::new(self.graph, from.clone(), to.clone(), handler)
                .search()
        } else {
            vec![]
        }
    }

    fn is_valid_index(&self, index: &GraphIndex) -> bool {
        self.is_valid_node(index) || self.graph.edge(index).is_some()
    }

    fn is_valid_node(&self, index: &GraphIndex) -> bool {
        self.graph.node(index).is_some()
    }
}
