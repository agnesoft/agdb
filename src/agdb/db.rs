pub mod db_element;
pub mod db_error;
pub mod db_index;
pub mod db_key;
pub mod db_key_value;
pub mod db_value;

mod db_float;

use crate::commands::insert_node::InsertNode;
use crate::commands::Commands;
use crate::graph::Graph;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::Query;
use crate::DbError;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

#[derive(Default)]
pub struct Db {
    pub graph: Graph,
}

impl Db {
    fn commands(query: &Query) -> Vec<Commands> {
        match query {
            Query::InsertAliases(query_data) => todo!(),
            Query::InsertEdges(query_data) => todo!(),
            Query::InsertNodes(query_data) => Self::insert_nodes(query_data),
            Query::InsertValues(query_data) => todo!(),
            Query::RemoveAliases(query_data) => todo!(),
            Query::Remove(query_data) => todo!(),
            Query::RemoveValues(query_data) => todo!(),
            Query::Search(query_data) => todo!(),
            Query::SelectAliases(query_data) => todo!(),
            Query::SelectKeys(query_data) => todo!(),
            Query::SelectKeyCount(query_data) => todo!(),
            Query::Select(query_data) => todo!(),
            Query::SelectValues(query_data) => todo!(),
        }
    }

    fn insert_nodes(query_data: &InsertNodesQuery) -> Vec<Commands> {
        let mut commands = Vec::<Commands>::new();

        for i in 0..query_data.count {
            commands.push(Commands::InsertNode(InsertNode {}));
        }

        commands
    }

    pub fn exec(&self, query: &Query) -> Result<QueryResult, QueryError> {
        let stack = Self::commands(query);

        Ok(QueryResult::default())
    }

    pub fn new(_filename: &str) -> Result<Db, DbError> {
        Ok(Self {
            graph: Graph::new(),
        })
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::default()
    }
}
