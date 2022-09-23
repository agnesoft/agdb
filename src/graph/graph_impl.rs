pub(super) struct GraphImpl {
    pub(super) from: Vec<i64>,
    pub(super) to: Vec<i64>,
    pub(super) from_meta: Vec<i64>,
    pub(super) to_meta: Vec<i64>,
    pub(super) node_count: u64,
}

impl GraphImpl {
    pub(super) fn first_edge_from(&self, index: i64) -> i64 {
        self.from[index as usize]
    }

    pub(super) fn next_edge_from(&self, index: i64) -> i64 {
        self.from_meta[(-index) as usize]
    }

    pub(super) fn next_node(&self, index: i64) -> Option<i64> {
        for i in (index as usize + 1)..self.from_meta.len() {
            if 0 <= self.from_meta[i] {
                return Some(i as i64);
            }
        }

        None
    }
}
