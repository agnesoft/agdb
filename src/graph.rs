use crate::DbError;

mod graph_edge;
mod graph_element;
mod graph_node;

pub(crate) struct Graph {
    from: Vec<i64>,
    to: Vec<i64>,
    from_meta: Vec<i64>,
    to_meta: Vec<i64>,
    node_count: u64,
}

#[allow(dead_code)]
impl Graph {
    pub(crate) fn new() -> Graph {
        Graph {
            from: vec![0],
            to: vec![0],
            from_meta: vec![0],
            to_meta: vec![0],
            node_count: 0,
        }
    }

    pub(crate) fn insert_edge(&mut self, from: i64, to: i64) -> Result<i64, DbError> {
        self.validate_node(from)?;
        self.validate_node(to)?;

        let index = self.from.len() as i64;

        self.from.push(from);
        let first_from = self.from[from as usize];
        self.from[from as usize] = index;
        self.from_meta.push(first_from);

        self.to.push(to);
        let first_to = self.to[to as usize];
        self.to[to as usize] = index;
        self.to_meta.push(first_to);

        Ok(-index)
    }

    pub(crate) fn insert_node(&mut self) -> i64 {
        self.from.push(0);
        self.to.push(0);
        self.from_meta.push(0);
        self.to_meta.push(0);
        self.node_count += 1;
        self.node_count as i64
    }

    fn validate_node(&self, index: i64) -> Result<(), DbError> {
        if let Some(meta) = self.from_meta.get(index as usize) {
            if 0 <= *meta {
                return Ok(());
            }
        }

        Err(DbError::from(format!("'{}' is not a valid node", index)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();

        assert_eq!(graph.insert_edge(from, to), Ok(-3_i64));
    }

    #[test]
    fn insert_edge_from_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let edge = graph.insert_edge(from, to).unwrap();

        assert_eq!(
            graph.insert_edge(edge, 2),
            Err(DbError::from(format!("'{}' is not a valid node", edge)))
        );
    }

    #[test]
    fn insert_edge_invalid_from() {
        let mut graph = Graph::new();

        assert_eq!(
            graph.insert_edge(1, 2),
            Err(DbError::from("'1' is not a valid node"))
        );
    }

    #[test]
    fn insert_edge_invalid_to() {
        let mut graph = Graph::new();
        let from = graph.insert_node();

        assert_eq!(
            graph.insert_edge(from, 2),
            Err(DbError::from("'2' is not a valid node"))
        );
    }

    #[test]
    fn insert_node() {
        let mut graph = Graph::new();
        let id = graph.insert_node();

        assert_eq!(id, 1);
    }
}
