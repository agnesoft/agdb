use agdb_graph::GraphIndex;

pub struct SearchData<'a> {
    pub distance: u64,
    pub index: GraphIndex,
    pub result: &'a mut Vec<GraphIndex>,
}
