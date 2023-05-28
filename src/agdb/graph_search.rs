pub mod path_search_handler;
pub mod search_control;
pub mod search_handler;

mod breadth_first_search;
mod breadth_first_search_reverse;
mod depth_first_search;
mod depth_first_search_reverse;
mod path;
mod path_search;
mod search_impl;
mod search_index;
mod search_iterator;

use self::breadth_first_search::BreadthFirstSearch;
use self::breadth_first_search_reverse::BreadthFirstSearchReverse;
use self::depth_first_search::DepthFirstSearch;
use self::depth_first_search_reverse::DepthFirstSearchReverse;
use self::path_search::PathSearch;
use self::path_search_handler::PathSearchHandler;
use self::search_handler::SearchHandler;
use self::search_impl::SearchImpl;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;

pub struct GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
}

#[allow(dead_code)]
impl<'a, Data> GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub fn breadth_first_search<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, BreadthFirstSearch>::new(self.graph, index).search(handler)
        } else {
            vec![]
        }
    }

    pub fn breadth_first_search_reverse<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, BreadthFirstSearchReverse>::new(self.graph, index)
                .search(handler)
        } else {
            vec![]
        }
    }

    pub fn depth_first_search<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, DepthFirstSearch>::new(self.graph, index).search(handler)
        } else {
            vec![]
        }
    }

    pub fn depth_first_search_reverse<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, DepthFirstSearchReverse>::new(self.graph, index).search(handler)
        } else {
            vec![]
        }
    }

    pub fn path<Handler: PathSearchHandler>(
        &self,
        from: GraphIndex,
        to: GraphIndex,
        handler: &'a Handler,
    ) -> Vec<GraphIndex> {
        if from != to && self.is_valid_node(from) && self.is_valid_node(to) {
            PathSearch::<'a, Data, Handler>::new(self.graph, from, to, handler).search()
        } else {
            vec![]
        }
    }

    fn is_valid_index(&self, index: GraphIndex) -> bool {
        self.is_valid_node(index) || self.graph.edge(index).is_some()
    }

    fn is_valid_node(&self, index: GraphIndex) -> bool {
        self.graph.node(index).is_some()
    }
}

impl<'a, Data> From<&'a GraphImpl<Data>> for GraphSearch<'a, Data>
where
    Data: GraphData,
{
    fn from(graph: &'a GraphImpl<Data>) -> Self {
        GraphSearch { graph }
    }
}
