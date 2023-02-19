use crate::graph::graph_index::GraphIndex;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {
    pub id: GraphIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            RemoveNode {
                id: GraphIndex { index: 0 }
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveNode {
                id: GraphIndex { index: 0 }
            },
            RemoveNode {
                id: GraphIndex { index: 0 }
            }
        );
    }
}
