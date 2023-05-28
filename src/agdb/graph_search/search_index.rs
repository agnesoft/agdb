use crate::graph::GraphIndex;

#[derive(Clone, Copy)]
pub(crate) struct SearchIndex {
    pub(crate) index: GraphIndex,
    pub(crate) distance: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let index = SearchIndex {
            index: GraphIndex(1),
            distance: 10,
        };
        let other = index.clone();

        assert_eq!(index.index, other.index);
        assert_eq!(index.distance, other.distance);
    }

    #[test]
    fn derived_from_copy() {
        let index = &SearchIndex {
            index: GraphIndex(1),
            distance: 10,
        };
        let other = *index;

        assert_eq!(index.index, other.index);
        assert_eq!(index.distance, other.distance);
    }
}
