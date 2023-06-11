mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryError;
use test_db::TestDb;

#[test]
fn insert_edges_from_to_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into()])
            .query(),
        1,
    );
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(&["alias1".into()])
                    .to(&[2.into()])
                    .query(),
            )?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(
        QueryBuilder::select().ids(&[(-3).into()]).query(),
        "Id '-3' not found",
    );
}

#[test]
fn insert_edges_missing_from() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into()])
            .query(),
        1,
    );
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .from(&["alias".into()])
            .to(&[2.into()])
            .query(),
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
                id: DbId(-5),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(-6),
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
                id: DbId(-5),
                values: vec![("key", "value1").into()],
            },
            DbElement {
                id: DbId(-6),
                values: vec![("key", "value2").into()],
            },
            DbElement {
                id: DbId(-7),
                values: vec![("key", "value3").into()],
            },
            DbElement {
                id: DbId(-8),
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
                id: DbId(-5),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-6),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-7),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-8),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_values_bad_length() {
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
                id: DbId(-4),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(-5),
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
                id: DbId(-4),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-5),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_search() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(4).query(), 4);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into(), 2.into()])
            .to(&[3.into(), 4.into()])
            .query(),
        2,
    );
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from_search(QueryBuilder::search().from(1).query())
            .to_search(QueryBuilder::search().from(3).query())
            .query(),
        &[-7, -8],
    );
}

#[test]
fn insert_edges_from_to_inserted_nodes() {
    let mut db = TestDb::new();
    let from = db.exec_mut_result(QueryBuilder::insert().nodes().count(2).query());
    let to = db.exec_mut_result(QueryBuilder::insert().nodes().count(2).query());

    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from(&from.ids())
            .to(&to.ids())
            .query(),
        &[-5, -6],
    );
}
