#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct GraphIndex {
    pub(crate) index: i64,
}

impl GraphIndex {
    pub fn as_u64(&self) -> u64 {
        if self.is_edge() {
            (-self.value()) as u64
        } else {
            self.value() as u64
        }
    }

    pub fn as_usize(&self) -> usize {
        if self.is_edge() {
            (-self.value()) as usize
        } else {
            self.value() as usize
        }
    }

    pub fn is_edge(&self) -> bool {
        self.index < 0
    }

    pub fn is_node(&self) -> bool {
        0 < self.index
    }

    pub fn is_valid(&self) -> bool {
        self.index != 0
    }

    pub fn value(&self) -> i64 {
        self.index
    }
}
