pub(super) struct GraphData {
    pub(super) from: Vec<i64>,
    pub(super) to: Vec<i64>,
    pub(super) from_meta: Vec<i64>,
    pub(super) to_meta: Vec<i64>,
    pub(super) node_count: u64,
}

impl GraphData {
    pub(super) fn capacity(&self) -> u64 {
        self.from.len() as u64
    }

    pub(super) fn free_index(&self) -> i64 {
        self.from_meta[0]
    }

    pub(super) fn from(&self, index: i64) -> i64 {
        self.from[index as usize]
    }

    pub(super) fn from_meta(&self, index: i64) -> i64 {
        self.from_meta[index as usize]
    }

    pub(super) fn node_count(&self) -> u64 {
        self.node_count
    }

    pub(super) fn resize(&mut self, capacity: u64) {
        self.from.resize(capacity as usize, 0);
        self.to.resize(capacity as usize, 0);
        self.from_meta.resize(capacity as usize, 0);
        self.to_meta.resize(capacity as usize, 0);
    }

    pub(super) fn set_from(&mut self, index: i64, value: i64) {
        self.from[index as usize] = value;
    }

    pub(super) fn set_from_meta(&mut self, index: i64, value: i64) {
        self.from_meta[index as usize] = value;
    }

    pub(super) fn set_node_count(&mut self, count: u64) {
        self.node_count = count
    }

    pub(super) fn set_to(&mut self, index: i64, value: i64) {
        self.to[index as usize] = value;
    }

    pub(super) fn set_to_meta(&mut self, index: i64, value: i64) {
        self.to_meta[index as usize] = value;
    }

    pub(super) fn to(&self, index: i64) -> i64 {
        self.to[index as usize]
    }

    pub(super) fn to_meta(&self, index: i64) -> i64 {
        self.to_meta[index as usize]
    }
}
