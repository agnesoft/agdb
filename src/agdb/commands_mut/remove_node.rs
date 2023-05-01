use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {
    id: DbId,
    alias: String,
    graph_index: GraphIndex,
    edges: Vec<Edge>,
}

#[derive(Debug, PartialEq)]
struct Edge {
    id: DbId,
    index: GraphIndex,
    from: GraphIndex,
    to: GraphIndex,
}

impl RemoveNode {
    pub(crate) fn new(id: QueryId) -> Self {
        match id {
            QueryId::Id(id) => Self {
                id,
                alias: String::new(),
                graph_index: GraphIndex { index: 0 },
                edges: vec![],
            },
            QueryId::Alias(alias) => Self {
                id: DbId(0),
                alias,
                graph_index: GraphIndex { index: 0 },
                edges: vec![],
            },
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        self.set_id_alias(db)?;

        if let Some(graph_index) = db.indexes.value(&self.id)? {
            self.remove_node(db, graph_index)?;
            result.result -= 1;
        }

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        if self.graph_index.is_valid() {
            let graph_index = db.graph.insert_node()?;
            db.indexes.insert(&self.id, &graph_index)?;

            if !self.alias.is_empty() {
                db.aliases.insert(&self.alias, &self.id)?;
            }

            for edge in self.edges {
                let graph_index = db.graph.insert_edge(&edge.from, &edge.to)?;
                db.indexes.insert(&edge.id, &graph_index)?;
            }
        }

        Ok(())
    }

    fn remove_node(&mut self, db: &mut Db, graph_index: GraphIndex) -> Result<(), QueryError> {
        let edges = edges(db, graph_index)?;
        self.edges.reserve(edges.len());

        for edge in edges {
            db.graph.remove_edge(&edge.index)?;
            db.indexes.remove_key(&edge.id)?;
            self.edges.push(edge);
        }

        db.graph.remove_node(&graph_index)?;
        self.graph_index = graph_index;
        db.aliases.remove_key(&self.alias)?;
        db.indexes.remove_key(&self.id)?;

        Ok(())
    }

    fn set_id_alias(&mut self, db: &mut Db) -> Result<(), QueryError> {
        if !self.alias.is_empty() {
            self.id = db.aliases.value(&self.alias)?.unwrap_or_default();
        } else {
            self.alias = db.aliases.key(&self.id)?.unwrap_or_default();
        }
        Ok(())
    }
}

fn edges(db: &Db, graph_index: GraphIndex) -> Result<Vec<Edge>, QueryError> {
    let node = db.graph.node(&graph_index).unwrap();
    let mut edges = vec![];

    for edge in node.edge_iter_from() {
        let id = db.indexes.key(&edge.index)?.unwrap_or_default();
        edges.push(Edge {
            id,
            index: edge.index,
            from: graph_index,
            to: edge.index_from(),
        });
    }

    for edge in node.edge_iter_to() {
        let from = edge.index_from();

        if from != graph_index {
            let id = db.indexes.key(&edge.index)?.unwrap_or_default();
            edges.push(Edge {
                id,
                index: edge.index,
                from,
                to: graph_index,
            });
        }
    }

    Ok(edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveNode::new("alias".into()));

        format!(
            "{:?}",
            Edge {
                id: DbId(0),
                index: GraphIndex { index: 0 },
                from: GraphIndex { index: 0 },
                to: GraphIndex { index: 0 }
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveNode::new("alias".into()),
            RemoveNode::new("alias".into())
        );

        assert_eq!(
            Edge {
                id: DbId(0),
                index: GraphIndex { index: 0 },
                from: GraphIndex { index: 0 },
                to: GraphIndex { index: 0 }
            },
            Edge {
                id: DbId(0),
                index: GraphIndex { index: 0 },
                from: GraphIndex { index: 0 },
                to: GraphIndex { index: 0 }
            }
        );
    }
}
