use crate::path::Path;
use crate::PathSearchHandler;
use agdb_bit_set::BitSet;
use agdb_graph::GraphData;
use agdb_graph::GraphImpl;
use agdb_graph::GraphIndex;
use std::cmp::Ordering;
use std::mem::swap;
use std::mem::take;

pub(crate) struct PathSearchImpl<'a, Data, Handler>
where
    Data: GraphData,
    Handler: PathSearchHandler,
{
    pub(crate) current_path: Path,
    pub(crate) destination: GraphIndex,
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) handler: &'a Handler,
    pub(crate) paths: Vec<Path>,
    pub(crate) result: Vec<GraphIndex>,
    pub(crate) visited: BitSet,
}

impl<'a, Data, Handler> PathSearchImpl<'a, Data, Handler>
where
    Data: GraphData,
    Handler: PathSearchHandler,
{
    pub(crate) fn new(
        graph: &'a GraphImpl<Data>,
        from: GraphIndex,
        to: GraphIndex,
        handler: &'a Handler,
    ) -> Self {
        Self {
            current_path: Path {
                elements: vec![],
                cost: 0,
            },
            destination: to,
            graph,
            handler,
            paths: vec![Path {
                elements: vec![from],
                cost: 0,
            }],
            result: vec![],
            visited: BitSet::new(),
        }
    }

    pub(crate) fn search(&mut self) -> Vec<GraphIndex> {
        while !self.is_finished() {
            self.sort_paths();
            self.process_last_path();
        }

        take(&mut self.result)
    }

    fn expand_edge(&mut self, mut path: Path, index: &GraphIndex, node_index: &GraphIndex) {
        let cost = self
            .handler
            .process(index, &(self.current_path.elements.len() as u64 + 1));

        if cost != 0 && !self.visited.value(node_index.as_u64()) {
            path.elements.push(index.clone());
            path.cost += cost;
            self.expand_node(path, node_index);
        }
    }

    fn expand_node(&mut self, mut path: Path, index: &GraphIndex) {
        let cost = self
            .handler
            .process(index, &(self.current_path.elements.len() as u64 + 1));

        if cost != 0 {
            path.elements.push(index.clone());
            path.cost += cost;
            self.paths.push(path);
        }
    }

    fn expand(&mut self, index: &GraphIndex) {
        let node = self
            .graph
            .node(index)
            .expect("unexpected invalid node index");
        for edge in node.edge_from_iter() {
            self.expand_edge(self.current_path.clone(), &edge.index(), &edge.to_index());
        }
    }

    fn is_finished(&self) -> bool {
        self.paths.is_empty() || !self.result.is_empty()
    }

    fn process_path(&mut self) {
        let index = self
            .current_path
            .elements
            .last()
            .map_or(GraphIndex::default(), |index| index.clone());
        self.process_index(&index);
    }

    fn process_index(&mut self, index: &GraphIndex) {
        if !self.visited.value(index.as_u64()) {
            if index.value() == self.destination.value() {
                swap(&mut self.result, &mut self.current_path.elements);
            } else {
                self.visited.insert(index.as_u64());
                self.expand(index);
            }
        }
    }

    fn process_last_path(&mut self) {
        self.current_path = self.paths.pop().unwrap_or(Path {
            elements: vec![],
            cost: 0,
        });
        self.process_path();
    }

    fn sort_paths(&mut self) {
        self.paths.sort_by(|left, right| {
            let ordering = left.cost.cmp(&right.cost);

            if ordering == Ordering::Equal {
                return left.elements.len().cmp(&right.elements.len()).reverse();
            }

            ordering.reverse()
        });
    }
}
