mod framework;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryError;
use framework::TestDb;

#[test]
fn insert_edge_from_to() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut_ids(
        QueryBuilder::insert().edge().from("alias1").to(2).query(),
        &[-3],
    );
}

#[test]
fn insert_edge_from_to_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::insert().edge().from("alias1").to(2).query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(QueryBuilder::select().id(-3).query(), "Id '-3' not found");
}

#[test]
fn insert_edge_missing_from() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut_error(
        QueryBuilder::insert().edge().from("alias").to(2).query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn insert_edges_from_to() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&[
                "alias1".into(),
                "alias2".into(),
                "alias3".into(),
                "alias4".into(),
            ])
            .query(),
        4,
    );
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into(), "alias4".into()])
            .each()
            .query(),
        &[-5, -6, -7, -8],
    );
}

#[test]
fn insert_edges_from_to_each() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into(), "alias2".into(), "alias3".into()])
            .query(),
        3,
    );
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into()])
            .query(),
        &[-4, -5],
    );
}

#[test]
fn insert_edges_from_to_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&[
                "alias1".into(),
                "alias2".into(),
                "alias3".into(),
                "alias4".into(),
            ])
            .query(),
        4,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into(), "alias4".into()])
            .values(&[&[("key", "value").into()], &[("key", "value2").into()]])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&[(-5).into(), (-6).into()])
            .query(),
        &[
            DbElement {
                index: DbId(-5),
                values: vec![("key", "value").into()],
            },
            DbElement {
                index: DbId(-6),
                values: vec![("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_each_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&[
                "alias1".into(),
                "alias2".into(),
                "alias3".into(),
                "alias4".into(),
            ])
            .query(),
        4,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into(), "alias4".into()])
            .each()
            .values(&[
                &[("key", "value1").into()],
                &[("key", "value2").into()],
                &[("key", "value3").into()],
                &[("key", "value4").into()],
            ])
            .query(),
        4,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&[(-5).into(), (-6).into(), (-7).into(), (-8).into()])
            .query(),
        &[
            DbElement {
                index: DbId(-5),
                values: vec![("key", "value1").into()],
            },
            DbElement {
                index: DbId(-6),
                values: vec![("key", "value2").into()],
            },
            DbElement {
                index: DbId(-7),
                values: vec![("key", "value3").into()],
            },
            DbElement {
                index: DbId(-8),
                values: vec![("key", "value4").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_each_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&[
                "alias1".into(),
                "alias2".into(),
                "alias3".into(),
                "alias4".into(),
            ])
            .query(),
        4,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into(), "alias4".into()])
            .each()
            .values_uniform(&[("key", "value").into(), ("key", "value2").into()])
            .query(),
        4,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&[(-5).into(), (-6).into(), (-7).into(), (-8).into()])
            .query(),
        &[
            DbElement {
                index: DbId(-5),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                index: DbId(-6),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                index: DbId(-7),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                index: DbId(-8),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_values_bad_length() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into(), "alias4".into()])
            .values(&[&[("key", "value").into()]])
            .query(),
        "Values len '1' do not match the insert count '2'",
    );
}

#[test]
fn insert_edges_from_to_values_each_bad_length() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into(), "alias4".into()])
            .each()
            .values(&[&[("key", "value").into()]])
            .query(),
        "Values len '1' do not match the insert count '4'",
    );
}

#[test]
fn insert_edges_from_to_values_asymmetric() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into(), "alias2".into(), "alias3".into()])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into()])
            .values(&[&[("key", "value").into()], &[("key", "value2").into()]])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&[(-4).into(), (-5).into()])
            .query(),
        &[
            DbElement {
                index: DbId(-4),
                values: vec![("key", "value").into()],
            },
            DbElement {
                index: DbId(-5),
                values: vec![("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into(), "alias2".into(), "alias3".into()])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias2".into()])
            .to(&["alias3".into()])
            .values_uniform(&[("key", "value").into(), ("key", "value2").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&[(-4).into(), (-5).into()])
            .query(),
        &[
            DbElement {
                index: DbId(-4),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                index: DbId(-5),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_query_to() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .from_search(QueryBuilder::search().from(1.into()).query())
            .to(&["alias".into()])
            .query(),
        "Invalid insert edges query",
    );
}

#[test]
fn insert_edge_from_to_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into(), "alias2".into()])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edge()
            .from("alias1")
            .to("alias2")
            .values(&[("key", "value").into()])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().id(-3).query(),
        &[DbElement {
            index: DbId(-3),
            values: vec![("key", "value").into()],
        }],
    );
}

#[test]
fn insert_edges_from_to_search() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .from(&["alias".into()])
            .to_search(QueryBuilder::search().from(2.into()).query())
            .query(),
        "Invalid insert edges query",
    );
}
