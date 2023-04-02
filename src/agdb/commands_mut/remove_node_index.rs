use super::insert_node::InsertNode;
use super::CommandsMut;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveNodeIndex {
    pub(crate) index: GraphIndex,
}

impl RemoveNodeIndex {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        db.graph.remove_node(&self.index)?;
        result.result -= 1;
        Ok(CommandsMut::InsertNode(InsertNode {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::graph_index::GraphIndex;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            RemoveNodeIndex {
                index: GraphIndex { index: 0 }
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveNodeIndex {
                index: GraphIndex { index: 0 }
            },
            RemoveNodeIndex {
                index: GraphIndex { index: 0 }
            }
        );
    }
}
