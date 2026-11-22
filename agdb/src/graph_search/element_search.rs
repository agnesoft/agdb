use crate::DbError;
use crate::DbId;
use crate::collections::graph::GraphHeader;
use crate::graph_search::SearchControl;
use crate::graph_search::SearchHandler;
use crate::storage::pages::View;

pub struct ElementSearch<'a, V: View, Handler: SearchHandler> {
    graph: &'a GraphHeader,
    view: &'a V,
    handler: Handler,
}

impl<'a, V: View, Handler: SearchHandler> ElementSearch<'a, V, Handler> {
    pub fn new(graph: &'a GraphHeader, view: &'a V, handler: Handler) -> Self {
        Self {
            graph,
            view,
            handler,
        }
    }

    pub fn search(&mut self) -> Result<Vec<DbId>, DbError> {
        let mut result = vec![];

        for (distance, id) in self.graph.elements(self.view).enumerate() {
            let id = id?;
            let (add, finished) = match self.handler.process(id, distance as u64)? {
                SearchControl::Continue(add) => (add, false),
                SearchControl::Finish(add) => (add, true),
                SearchControl::Stop(add) => (add, false),
            };

            if add {
                result.push(id);
            }

            if finished {
                break;
            }
        }

        Ok(result)
    }
}
