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

impl From<i64> for GraphIndex {
    fn from(index: i64) -> Self {
        Self { index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn derived_from_debug() {
        let index = GraphIndex::default();

        format!("{:?}", index);
    }

    #[test]
    fn derived_from_ord() {
        let index = GraphIndex::default();
        assert_eq!(index.cmp(&index), Ordering::Equal);
    }

    #[test]
    fn is_edge() {
        assert!(!GraphIndex::from(1).is_edge());
        assert!(!GraphIndex::default().is_edge());
        assert!(GraphIndex::from(-1).is_edge());
    }

    #[test]
    fn is_node() {
        assert!(GraphIndex::from(1).is_node());
        assert!(!GraphIndex::default().is_node());
        assert!(!GraphIndex::from(-1).is_node());
    }

    #[test]
    fn ordering() {
        let mut indexes = vec![
            GraphIndex::default(),
            GraphIndex::from(100_i64),
            GraphIndex::from(-1_i64),
            GraphIndex::from(1_i64),
        ];

        indexes.sort();

        assert_eq!(
            indexes,
            vec![
                GraphIndex::from(-1_i64),
                GraphIndex::default(),
                GraphIndex::from(1_i64),
                GraphIndex::from(100_i64),
            ]
        )
    }
}
