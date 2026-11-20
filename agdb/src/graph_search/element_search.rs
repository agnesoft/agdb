use std::mem::swap;

use crate::DbError;
use crate::StorageData;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::graph_search::SearchControl;
use crate::graph_search::SearchHandler;
use crate::storage::Storage;

pub struct ElementSearch<'a, D, Handler>
where
    D: StorageData,
    Handler: SearchHandler,
{
    graph: &'a DbGraph<D>,
    storage: &'a Storage<D>,
    handler: Handler,
    result: Vec<GraphIndex>,
}

impl<'a, D, Handler> ElementSearch<'a, D, Handler>
where
    D: StorageData,
    Handler: SearchHandler,
{
    pub fn new(graph: &'a DbGraph<D>, storage: &'a Storage<D>, handler: Handler) -> Self {
        Self {
            graph,
            storage,
            handler,
            result: vec![],
        }
    }

    pub fn search(&mut self) -> Result<Vec<GraphIndex>, DbError> {
        for (distance, index) in self.graph.iter(self.storage).enumerate() {
            let (add_index, finished) = match self.handler.process(index, distance as u64)? {
                SearchControl::Continue(add) => (add, false),
                SearchControl::Finish(add) => (add, true),
                SearchControl::Stop(add) => (add, false),
            };

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
