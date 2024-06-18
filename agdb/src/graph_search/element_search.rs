use std::mem::swap;

use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use crate::graph_search::SearchControl;
use crate::graph_search::SearchHandler;
use crate::storage::Storage;
use crate::DbError;
use crate::StorageData;

pub struct ElementSearch<'a, D, Data, Handler>
where
    Data: GraphData<D>,
    D: StorageData,
    Handler: SearchHandler,
{
    graph: &'a GraphImpl<D, Data>,
    storage: &'a Storage<D>,
    handler: Handler,
    result: Vec<GraphIndex>,
}

impl<'a, D, Data, Handler> ElementSearch<'a, D, Data, Handler>
where
    Data: GraphData<D>,
    D: StorageData,
    Handler: SearchHandler,
{
    pub fn new(graph: &'a GraphImpl<D, Data>, storage: &'a Storage<D>, handler: Handler) -> Self {
        Self {
            graph,
            storage,
            handler,
            result: vec![],
        }
    }

    pub fn search(&mut self) -> Result<Vec<GraphIndex>, DbError> {
        for (distance, index) in self.graph.iter(self.storage).enumerate() {
            let add_index;
            let finished;

            match self.handler.process(index, distance as u64)? {
                SearchControl::Continue(add) => {
                    add_index = add;
                    finished = false;
                }
                SearchControl::Finish(add) => {
                    add_index = add;
                    finished = true;
                }
                SearchControl::Stop(add) => {
                    add_index = add;
                    finished = false;
                }
            }

            if add_index {
                self.result.push(index);
            }

            if finished {
                break;
            }
        }

        Ok(self.take_result())
    }

    fn take_result(&mut self) -> Vec<GraphIndex> {
        let mut res = Vec::<GraphIndex>::new();
        swap(&mut res, &mut self.result);

        res
    }
}
