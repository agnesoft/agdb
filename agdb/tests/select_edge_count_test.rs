mod test_db;

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
            .aliases(["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_count_elements(
        QueryBuilder::select().edge_count().ids("from").query(),
        1,
        &[DbElement {
            id: DbId(1),
            from: DbId(-3),
            to: DbId::default(),
            values: vec![("edge_count", 1_u64).into()],
        }],
    );

    db.exec_count_elements(
        QueryBuilder::select().edge_count().ids("to").query(),
        1,
        &[DbElement {
            id: DbId(2),
            from: DbId::default(),
            to: DbId(-3),
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
            .aliases(["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_count_elements(
        QueryBuilder::select().edge_count_from().ids("from").query(),
        1,
        &[DbElement {
            id: DbId(1),
            from: DbId(-3),
            to: DbId::default(),
            values: vec![("edge_count", 1_u64).into()],
        }],
    );

    db.exec_count_elements(
        QueryBuilder::select().edge_count_from().ids("to").query(),
        0,
        &[DbElement {
            id: DbId(2),
            from: DbId::default(),
            to: DbId(-3),
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
            .aliases(["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_count_elements(
        QueryBuilder::select().edge_count_to().ids("from").query(),
        0,
        &[DbElement {
            id: DbId(1),
            from: DbId(-3),
            to: DbId::default(),
            values: vec![("edge_count", 0_u64).into()],
        }],
    );

    db.exec_count_elements(
        QueryBuilder::select().edge_count_to().ids("to").query(),
        1,
        &[DbElement {
            id: DbId(2),
            from: DbId::default(),
            to: DbId(-3),
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
            .aliases(["node1", "node2", "node3"])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["node1", "node2", "node1"])
            .to(["node2", "node3", "node3"])
            .query(),
        3,
    );

    db.exec_count_elements(
        QueryBuilder::select()
            .edge_count()
            .ids(["node1", "node2", "node3"])
            .query(),
        6,
        &[
            DbElement {
                id: DbId(1),
                from: DbId(-6),
                to: DbId::default(),
                values: vec![("edge_count", 2_u64).into()],
            },
            DbElement {
                id: DbId(2),
                from: DbId(-5),
                to: DbId(-4),
                values: vec![("edge_count", 2_u64).into()],
            },
            DbElement {
                id: DbId(3),
                from: DbId::default(),
                to: DbId(-6),
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
            .aliases(["node1", "node2", "node3"])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["node1", "node3", "node2", "node2"])
            .to(["node3", "node2", "node1", "node2"])
            .query(),
        4,
    );

    db.exec_count_elements(
        QueryBuilder::select()
            .edge_count()
            .ids(
                QueryBuilder::search()
                    .from("node1")
                    .where_()
                    .edge_count(4)
                    .query(),
            )
            .query(),
        4,
        &[DbElement {
            id: DbId(2),
            from: DbId(-7),
            to: DbId(-7),
            values: vec![("edge_count", 4_u64).into()],
        }],
    );
}

#[test]
fn select_edge_count_search_alt() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["node1", "node2", "node3"])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["node1", "node3", "node2", "node2"])
            .to(["node3", "node2", "node1", "node2"])
            .query(),
        4,
    );

    db.exec_count_elements(
        QueryBuilder::select()
            .edge_count()
            .search()
            .from("node1")
            .where_()
            .edge_count(4)
            .query(),
        4,
        &[DbElement {
            id: DbId(2),
            from: DbId(-7),
            to: DbId(-7),
            values: vec![("edge_count", 4_u64).into()],
        }],
    );

    db.exec_count_elements(
        QueryBuilder::select()
            .edge_count_from()
            .search()
            .from("node1")
            .where_()
            .edge_count(4)
            .query(),
        2,
        &[DbElement {
            id: DbId(2),
            from: DbId(-7),
            to: DbId(-7),
            values: vec![("edge_count", 2_u64).into()],
        }],
    );

    db.exec_count_elements(
        QueryBuilder::select()
            .edge_count_to()
            .search()
            .from("node1")
            .where_()
            .edge_count(4)
            .query(),
        2,
        &[DbElement {
            id: DbId(2),
            from: DbId(-7),
            to: DbId(-7),
            values: vec![("edge_count", 2_u64).into()],
        }],
    );
}

#[test]
fn select_edge_count_non_nodes() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_count_elements(
        QueryBuilder::select().edge_count().ids(-3).query(),
        0,
        &[DbElement {
            id: DbId(-3),
            from: DbId(1),
            to: DbId(2),
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
            .aliases(["from", "to"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert().edges().from("from").to("to").query(),
        1,
    );

    db.exec_count_elements(
        SelectEdgeCountQuery {
            ids: "from".into(),
            from: false,
            to: false,
        },
        0,
        &[DbElement {
            id: DbId(1),
            from: DbId(-3),
            to: DbId::default(),
            values: vec![("edge_count", 0_u64).into()],
        }],
    );
}
