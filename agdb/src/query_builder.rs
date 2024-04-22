mod insert;
mod insert_aliases;
mod insert_edge;
mod insert_index;
mod insert_nodes;
mod insert_values;
mod remove;
mod remove_aliases;
mod remove_ids;
mod remove_index;
mod remove_values;
mod search;
mod select;
mod select_aliases;
mod select_edge_count;
mod select_ids;
mod select_indexes;
mod select_key_count;
mod select_keys;
mod select_values;
mod where_;

use self::insert::Insert;
use self::remove::Remove;
use self::search::Search;
use self::select::Select;

/// The starting point of all queries.
///
/// Options:
///
/// ```
/// use agdb::QueryBuilder;
///
/// QueryBuilder::insert();
/// QueryBuilder::remove();
/// QueryBuilder::search();
/// QueryBuilder::select();
/// ```
pub struct QueryBuilder {}

impl QueryBuilder {
    /// Allows inserting data into the database:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{DbId, QueryBuilder, UserValue};
    ///
    /// #[derive(UserValue)]
    /// struct MyValue { db_id: Option<DbId>, key: String }
    ///
    /// QueryBuilder::insert().nodes();
    /// QueryBuilder::insert().edges();
    /// QueryBuilder::insert().aliases("a");
    /// QueryBuilder::insert().aliases(vec!["a", "b"]);
    /// QueryBuilder::insert().element(&MyValue { db_id: Some(DbId(1)), key: "a".to_string(), });
    /// QueryBuilder::insert().elements(&[MyValue { db_id: Some(DbId(1)), key: "a".to_string(), }]);
    /// QueryBuilder::insert().index("k");
    /// QueryBuilder::insert().values(vec![vec![("k", 1).into()]]);
    /// QueryBuilder::insert().values_uniform(vec![("k", 1).into()]);
    /// ```
    pub fn insert() -> Insert {
        Insert {}
    }

    /// Allows removing data from the database:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::remove().ids(1);
    /// QueryBuilder::remove().ids(vec![1, 2]);
    /// QueryBuilder::remove().ids(QueryBuilder::search().from(1).query());
    /// QueryBuilder::remove().index("k");
    /// QueryBuilder::remove().aliases("a");
    /// QueryBuilder::remove().aliases(vec!["a", "b"]);
    /// QueryBuilder::remove().values(vec!["k".into()]);
    /// ```
    pub fn remove() -> Remove {
        Remove {}
    }

    /// Search the database by traversing the graph
    /// and returns element ids using breadth first,
    /// depth first or A* algorithm:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().from(1); // BDS
    /// QueryBuilder::search().to(1); // BDS
    /// QueryBuilder::search().breadth_first();
    /// QueryBuilder::search().depth_first();
    /// ```
    pub fn search() -> Search {
        Search {}
    }

    /// Selects data from the database:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::select().ids(1);
    /// QueryBuilder::select().ids(vec![1, 2]);
    /// QueryBuilder::select().ids(QueryBuilder::search().from(1).query());
    /// QueryBuilder::select().aliases();
    /// QueryBuilder::select().keys();
    /// QueryBuilder::select().key_count();
    /// QueryBuilder::select().values(vec!["k".into()]);
    /// ```
    pub fn select() -> Select {
        Select {}
    }
}
