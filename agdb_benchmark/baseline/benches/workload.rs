// Compiled once per agdb version (see baseline.rs), so it uses only the API
// both versions share.

use agdb::Comparison;
use agdb::DbId;
use agdb::DbImpl;
use agdb::DbKeyOrder;
use agdb::DbKeyValue;
use agdb::QueryBuilder;
use agdb::StorageData;
use std::path::Path;

pub fn values(i: u64) -> Vec<DbKeyValue> {
    vec![
        ("name", format!("node{i}")).into(),
        ("k", i).into(),
        ("tag", "abc").into(),
    ]
}

/// The `root` node and `nodes` nodes with values, each one linked from
/// `root`, with an index on `k` when `index`.
pub fn create<S: StorageData>(path: &Path, nodes: u64, index: bool) -> DbImpl<S> {
    let mut db = DbImpl::new(path.to_str().expect("temp dir should be valid UTF-8"))
        .expect("db should be created");

    if index {
        db.exec_mut(QueryBuilder::insert().index("k").query())
            .unwrap();
    }

    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    db.transaction_mut(|t| -> Result<(), agdb::DbError> {
        let values: Vec<Vec<DbKeyValue>> = (0..nodes).map(values).collect();
        let ids = t
            .exec_mut(QueryBuilder::insert().nodes().values(values).query())?
            .ids();
        t.exec_mut(QueryBuilder::insert().edges().from("root").to(ids).query())?;
        Ok(())
    })
    .unwrap();
    db
}

pub fn open<S: StorageData>(path: &Path) -> DbImpl<S> {
    DbImpl::new(path.to_str().expect("temp dir should be valid UTF-8")).expect("db should open")
}

/// A node id among the `nodes` created ones, the first one is 2 (1 is root).
pub fn id(counter: u64, nodes: u64) -> DbId {
    DbId(2 + (counter % nodes) as i64)
}

pub fn insert_node<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, _n: u64) -> u64 {
    *counter += 1;
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([values(*counter)])
            .query(),
    )
    .unwrap()
    .result
}

pub fn insert_edge<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, nodes: u64) -> u64 {
    *counter += 1;
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from("root")
            .to(id(*counter, nodes))
            .query(),
    )
    .unwrap()
    .result
}

pub fn update_value<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, nodes: u64) -> u64 {
    *counter += 1;
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("k", *counter).into()]])
            .ids(id(*counter, nodes))
            .query(),
    )
    .unwrap()
    .result
}

pub fn update_value_resize<S: StorageData>(
    db: &mut DbImpl<S>,
    counter: &mut u64,
    nodes: u64,
) -> u64 {
    *counter += 1;
    let name = "x".repeat(16 + (*counter % 64) as usize);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("name", name).into()]])
            .ids(id(*counter, nodes))
            .query(),
    )
    .unwrap()
    .result
}

pub fn insert_remove_node<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, _n: u64) -> u64 {
    *counter += 1;
    let ids = db
        .exec_mut(
            QueryBuilder::insert()
                .nodes()
                .values([values(*counter)])
                .query(),
        )
        .unwrap()
        .ids();
    db.exec_mut(QueryBuilder::remove().ids(ids).query())
        .unwrap()
        .result
}

pub fn insert_large_value<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, size: u64) -> u64 {
    *counter += 1;
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("blob", vec![(*counter % 256) as u8; size as usize]).into()]])
            .query(),
    )
    .unwrap()
    .result
}

pub fn transaction<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, count: u64) -> u64 {
    *counter += count;
    let start = *counter;
    db.transaction_mut(|t| -> Result<(), agdb::DbError> {
        let nodes: Vec<Vec<DbKeyValue>> = (start..start + count).map(values).collect();
        let ids = t
            .exec_mut(QueryBuilder::insert().nodes().values(nodes).query())?
            .ids();
        t.exec_mut(QueryBuilder::insert().edges().from("root").to(ids).query())?;
        Ok(())
    })
    .unwrap();
    count
}

pub fn insert_nodes<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, count: u64) -> u64 {
    *counter += count;
    let start = *counter;
    let nodes: Vec<Vec<DbKeyValue>> = (start..start + count).map(values).collect();
    db.exec_mut(QueryBuilder::insert().nodes().values(nodes).query())
        .unwrap()
        .result
}

pub fn insert_remove_nodes<S: StorageData>(
    db: &mut DbImpl<S>,
    counter: &mut u64,
    count: u64,
) -> u64 {
    *counter += count;
    let start = *counter;
    let nodes: Vec<Vec<DbKeyValue>> = (start..start + count).map(values).collect();
    let ids = db
        .exec_mut(QueryBuilder::insert().nodes().values(nodes).query())
        .unwrap()
        .ids();
    db.exec_mut(QueryBuilder::remove().ids(ids).query())
        .unwrap()
        .result
}

pub fn select_element<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, nodes: u64) -> u64 {
    *counter += 1;
    db.exec(QueryBuilder::select().ids(id(*counter, nodes)).query())
        .unwrap()
        .result
}

pub fn select_value<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, nodes: u64) -> u64 {
    *counter += 1;
    db.exec(
        QueryBuilder::select()
            .values("k")
            .ids(id(*counter, nodes))
            .query(),
    )
    .unwrap()
    .result
}

pub fn select_alias<S: StorageData>(db: &mut DbImpl<S>, _counter: &mut u64, _n: u64) -> u64 {
    db.exec(QueryBuilder::select().ids("root").query())
        .unwrap()
        .result
}

pub fn search_index<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64, nodes: u64) -> u64 {
    *counter += 1;
    db.exec(
        QueryBuilder::search()
            .index("k")
            .value(*counter % nodes)
            .query(),
    )
    .unwrap()
    .result
}

/// All nodes linked from `root` whose `k` is below `limit`, the elements
/// and the edges from `root` are all examined.
pub fn search_condition<S: StorageData>(db: &mut DbImpl<S>, _counter: &mut u64, limit: u64) -> u64 {
    db.exec(
        QueryBuilder::search()
            .from("root")
            .where_()
            .node()
            .and()
            .key("k")
            .value(Comparison::LessThan(limit.into()))
            .query(),
    )
    .unwrap()
    .result
}

/// The first `limit` nodes from `root` ordered by `k` descending.
pub fn search_order_by<S: StorageData>(db: &mut DbImpl<S>, _counter: &mut u64, limit: u64) -> u64 {
    db.exec(
        QueryBuilder::search()
            .from("root")
            .order_by(DbKeyOrder::Desc("k".into()))
            .limit(limit)
            .where_()
            .node()
            .query(),
    )
    .unwrap()
    .result
}

/// The values of every node linked from `root`.
pub fn select_search_values<S: StorageData>(
    db: &mut DbImpl<S>,
    _counter: &mut u64,
    _n: u64,
) -> u64 {
    db.exec(
        QueryBuilder::select()
            .values(["name", "k"])
            .search()
            .from("root")
            .where_()
            .node()
            .query(),
    )
    .unwrap()
    .result
}
