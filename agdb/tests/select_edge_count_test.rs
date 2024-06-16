mod test_db;

use agdb::CountComparison;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::SelectEdgeCountQuery;
use test_db::TestDb;

#[test]
fn select_edge_count_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_elements(
        QueryBuilder::select().edge_count().ids("from").query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("edge_count", 1_u64).into()],
        }],
    );

    db.exec_elements(
        QueryBuilder::select().edge_count().ids("to").query(),
        &[DbElement {
            id: DbId(2),
            from: None,
            to: None,
            values: vec![("edge_count", 1_u64).into()],
        }],
    );
}

#[test]
fn select_edge_count_from_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_elements(
        QueryBuilder::select().edge_count_from().ids("from").query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("edge_count", 1_u64).into()],
        }],
    );

    db.exec_elements(
        QueryBuilder::select().edge_count_from().ids("to").query(),
        &[DbElement {
            id: DbId(2),
            from: None,
            to: None,
            values: vec![("edge_count", 0_u64).into()],
        }],
    );
}

#[test]
fn select_edge_count_to_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_elements(
        QueryBuilder::select().edge_count_to().ids("from").query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("edge_count", 0_u64).into()],
        }],
    );

    db.exec_elements(
        QueryBuilder::select().edge_count_to().ids("to").query(),
        &[DbElement {
            id: DbId(2),
            from: None,
            to: None,
            values: vec![("edge_count", 1_u64).into()],
        }],
    );
}

#[test]
fn select_edge_count_multi() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["node1", "node2", "node3"])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec!["node1", "node2", "node1"])
            .to(vec!["node2", "node3", "node3"])
            .query(),
        3,
    );

    db.exec_elements(
        QueryBuilder::select()
            .edge_count()
            .ids(vec!["node1", "node2", "node3"])
            .query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("edge_count", 2_u64).into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("edge_count", 2_u64).into()],
            },
            DbElement {
                id: DbId(3),
                from: None,
                to: None,
                values: vec![("edge_count", 2_u64).into()],
            },
        ],
    );
}

#[test]
fn select_edge_count_search() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["node1", "node2", "node3"])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec!["node1", "node3", "node2", "node2"])
            .to(vec!["node3", "node2", "node1", "node2"])
            .query(),
        4,
    );

    db.exec_elements(
        QueryBuilder::select()
            .edge_count()
            .ids(
                QueryBuilder::search()
                    .from("node1")
                    .where_()
                    .edge_count(CountComparison::Equal(4))
                    .query(),
            )
            .query(),
        &[DbElement {
            id: DbId(2),
            from: None,
            to: None,
            values: vec![("edge_count", 4_u64).into()],
        }],
    );
}

#[test]
fn select_edge_count_non_nodes() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_elements(
        QueryBuilder::select().edge_count().ids(-3).query(),
        &[DbElement {
            id: DbId(-3),
            from: Some(DbId(1)),
            to: Some(DbId(2)),
            values: vec![("edge_count", 0_u64).into()],
        }],
    );
}

#[test]
fn select_edge_count_invalid_query() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_elements(
        SelectEdgeCountQuery {
            ids: "from".into(),
            from: false,
            to: false,
        },
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("edge_count", 0_u64).into()],
        }],
    );
}
