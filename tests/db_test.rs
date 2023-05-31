#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_file::TestFile;

#[test]
fn data_persistence() {
    let test_file = TestFile::new();
    let values = vec![
        ("key", "String that is much longer than 15 characters").into(),
        (-10_i64, 2000000000000_i64).into(),
        (10_u64, 1.1_f64).into(),
        (vec!["Some", "List"], vec![1_u64, 2_u64]).into(),
        (
            vec![-1_i64, -2_i64, -3_i64],
            vec![-3.3_f64, -3.4_f64, -720.984_f64],
        )
            .into(),
        (vec![3_u8; 5], vec![15_u8; 20]).into(),
    ];

    {
        let mut db = Db::new(test_file.file_name()).unwrap();
        db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(&["alias".into(), "alias2".into()])
                .values_uniform(&values)
                .query(),
        )
        .unwrap();
        db.exec_mut(&QueryBuilder::insert().edge().from(1).to(2).query())
            .unwrap();
        let result = db
            .exec(
                &QueryBuilder::select()
                    .ids(&["alias".into(), "alias2".into(), (-3).into()])
                    .query(),
            )
            .unwrap();

        assert_eq!(
            result.elements,
            &[
                DbElement {
                    index: DbId(1),
                    values: values.clone(),
                },
                DbElement {
                    index: DbId(2),
                    values: values.clone(),
                },
                DbElement {
                    index: DbId(-3),
                    values: vec![],
                }
            ]
        );
    }

    let db = Db::new(test_file.file_name()).unwrap();
    let result = db
        .exec(
            &QueryBuilder::select()
                .ids(&["alias".into(), "alias2".into(), (-3).into()])
                .query(),
        )
        .unwrap();

    assert_eq!(
        result.elements,
        &[
            DbElement {
                index: DbId(1),
                values: values.clone(),
            },
            DbElement {
                index: DbId(2),
                values: values.clone(),
            },
            DbElement {
                index: DbId(-3),
                values: vec![],
            }
        ]
    );
}

#[test]
fn data_remove_persistence() {
    let test_file = TestFile::new();

    {
        let mut db = Db::new(test_file.file_name()).unwrap();
        db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(&["alias".into(), "alias2".into()])
                .values_uniform(&[("key", 100).into()])
                .query(),
        )
        .unwrap();
        db.exec_mut(&QueryBuilder::insert().edge().from(1).to(2).query())
            .unwrap();
        let result = db
            .exec(
                &QueryBuilder::select()
                    .ids(&["alias".into(), "alias2".into(), (-3).into()])
                    .query(),
            )
            .unwrap();

        assert_eq!(
            result.elements,
            &[
                DbElement {
                    index: DbId(1),
                    values: vec![("key", 100).into()],
                },
                DbElement {
                    index: DbId(2),
                    values: vec![("key", 100).into()],
                },
                DbElement {
                    index: DbId(-3),
                    values: vec![],
                }
            ]
        );

        db.exec_mut(&QueryBuilder::remove().id(-3).query()).unwrap();
        db.exec_mut(&QueryBuilder::remove().value("key").id(1).query())
            .unwrap();
    }

    let db = Db::new(test_file.file_name()).unwrap();
    let result = db
        .exec(
            &QueryBuilder::select()
                .ids(&["alias".into(), "alias2".into()])
                .query(),
        )
        .unwrap();

    assert_eq!(
        result.elements,
        &[
            DbElement {
                index: DbId(1),
                values: vec![],
            },
            DbElement {
                index: DbId(2),
                values: vec![("key", 100).into()],
            }
        ]
    );

    let error = db.exec(&QueryBuilder::select().id(-3).query()).unwrap_err();
    assert_eq!(error.description, "Id '-3' not found");
}

#[test]
fn invalid_db_file() {
    let test_file = TestFile::new();
    let bytes = [0_u64.to_le_bytes(), 0_u64.to_le_bytes()].concat();
    std::fs::write(test_file.file_name(), bytes).unwrap();
    let error = Db::new(test_file.file_name()).unwrap_err();

    assert_eq!(error.description, "Failed to create database");
    assert_eq!(
        error.cause.unwrap().description,
        format!(
            "File '{}' is not a valid database file and is not empty.",
            test_file.file_name()
        )
    );
}
