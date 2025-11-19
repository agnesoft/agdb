use crate::test_db::TestDb;
use agdb::AgdbSerialize;
use agdb::DbElement;
use agdb::DbError;
use agdb::DbId;
use agdb::DbSerialize;
use agdb::DbType;
use agdb::DbTypeMarker;
use agdb::DbValue;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::QueryResult;

#[derive(Default, Debug, Clone, PartialEq, DbTypeMarker, DbValue, DbSerialize)]
enum Status {
    Active,
    #[default]
    Inactive,
}

#[derive(DbType, Debug)]
struct User {
    user_id: u64,
    password: String,
    status: Status,
}

#[derive(DbType, PartialEq, Debug)]
struct MyValue {
    db_id: Option<QueryId>,
    name: String,
    age: u64,
}

#[derive(DbType, PartialEq, Debug)]
struct MyValueWithBool {
    db_id: Option<QueryId>,
    name: String,
    is_active: bool,
    truths: Vec<bool>,
}

#[derive(Clone, PartialEq, Debug, DbValue, DbTypeMarker, DbSerialize)]
struct Attribute {
    name: String,
    value: String,
}

#[derive(DbType, Default, PartialEq, Debug)]
struct MyCustomVec {
    vec: Vec<Status>,
    attributes: Vec<Attribute>,
}

#[derive(DbType, PartialEq, Debug)]
struct WithOption {
    name: String,
    value: Option<u64>,
}

#[test]
fn user_db_types() {
    #[derive(Default, Debug, PartialEq, DbType)]
    struct MyData {
        bytes: Vec<u8>,
        u64: u64,
        u32: u32,
        i64: i64,
        i32: i32,
        f64: f64,
        f32: f32,
        string: String,
        vec_u64: Vec<u64>,
        vec_u32: Vec<u32>,
        vec_i64: Vec<i64>,
        vec_i32: Vec<i32>,
        vec_f64: Vec<f64>,
        vec_f32: Vec<f32>,
        vec_string: Vec<String>,
        custom_enum: Status,
    }

    let my_data = MyData {
        bytes: vec![1_u8],
        u64: 1_u64,
        u32: 2_u32,
        i64: -1_i64,
        i32: -2_i32,
        f64: 1.1_f64,
        f32: 2.2_f32,
        string: "hello".to_string(),
        vec_u64: vec![1_u64],
        vec_u32: vec![2_u32],
        vec_i64: vec![-1_i64],
        vec_i32: vec![-2_i32],
        vec_f64: vec![1.1_f64],
        vec_f32: vec![2.2_f32],
        vec_string: vec!["world".to_string()],
        custom_enum: Status::Active,
    };
    let keys: Vec<DbValue> = vec![
        "bytes".into(),
        "u64".into(),
        "u32".into(),
        "i64".into(),
        "i32".into(),
        "f64".into(),
        "f32".into(),
        "string".into(),
        "vec_u64".into(),
        "vec_u32".into(),
        "vec_i64".into(),
        "vec_i32".into(),
        "vec_f64".into(),
        "vec_f32".into(),
        "vec_string".into(),
        "custom_enum".into(),
    ];

    let element = DbElement {
        id: DbId(0),
        from: None,
        to: None,
        values: vec![
            ("bytes", vec![1_u8]).into(),
            ("u64", 1_u64).into(),
            ("u32", 2_u64).into(),
            ("i64", -1_i64).into(),
            ("i32", -2_i64).into(),
            ("f64", 1.1_f64).into(),
            ("f32", 2.2_f32).into(),
            ("string", "hello").into(),
            ("vec_u64", vec![1_u64]).into(),
            ("vec_u32", vec![2_u32]).into(),
            ("vec_i64", vec![-1_i64]).into(),
            ("vec_i32", vec![-2_i32]).into(),
            ("vec_f64", vec![1.1_f64]).into(),
            ("vec_f32", vec![2.2_f32]).into(),
            ("vec_string", vec!["world"]).into(),
            ("custom_enum", Status::Active).into(),
        ],
    };

    assert_eq!(MyData::db_keys(), keys);
    assert_eq!(&my_data.to_db_values(), &element.values);
    assert_eq!(MyData::from_db_element(&element).unwrap(), my_data);
}

#[test]
fn insert_node_values_custom() {
    #[derive(DbType)]
    struct MyValue {
        name: String,
        age: u64,
    }

    let mut db = TestDb::new();
    let my_value = MyValue {
        name: "my name".to_string(),
        age: 20,
    };
    db.exec_mut(QueryBuilder::insert().nodes().values(&my_value).query(), 1);
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("name", "my name").into(), ("age", 20_u64).into()],
        }],
    );
}

#[test]
fn insert_node_values_uniform_custom() {
    #[derive(DbType)]
    struct MyValue {
        name: String,
        age: u64,
    }

    let mut db = TestDb::new();
    let my_value = MyValue {
        name: "my name".to_string(),
        age: 20,
    };
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(2)
            .values_uniform(&my_value)
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids([1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("name", "my name").into(), ("age", 20_u64).into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("name", "my name").into(), ("age", 20_u64).into()],
            },
        ],
    );
}

#[test]
fn select_custom_value_keys() {
    #[derive(Debug, Clone, PartialEq, DbType)]
    struct MyValue {
        name: String,
        age: u64,
    }

    let mut db = TestDb::new();
    let my_value = MyValue {
        name: "my name".to_string(),
        age: 20,
    };
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(2)
            .values_uniform(&my_value)
            .query(),
        2,
    );

    let db_value: MyValue = db
        .exec_result(
            QueryBuilder::select()
                .values(MyValue::db_keys())
                .ids(1)
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(&my_value, &db_value);

    let db_values: Vec<MyValue> = db
        .exec_result(
            QueryBuilder::select()
                .values(MyValue::db_keys())
                .ids([1, 2])
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(db_values, vec![my_value.clone(), my_value]);
}

#[test]
fn select_custom_value_with_id() {
    #[derive(Debug, Clone, PartialEq, DbType)]
    struct MyValue {
        db_id: Option<DbId>,
        name: String,
        age: u64,
    }

    let mut db = TestDb::new();
    let my_value = MyValue {
        db_id: None,
        name: "my name".to_string(),
        age: 20,
    };

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(2)
            .values_uniform(&my_value)
            .query(),
        2,
    );

    let db_values: Vec<MyValue> = db
        .exec_result(
            QueryBuilder::select()
                .values(MyValue::db_keys())
                .ids([1, 2])
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(
        db_values,
        vec![
            MyValue {
                db_id: Some(DbId(1)),
                name: "my name".to_string(),
                age: 20
            },
            MyValue {
                db_id: Some(DbId(2)),
                name: "my name".to_string(),
                age: 20
            }
        ]
    );
}

#[test]
fn insert_single_element() {
    #[derive(Debug, Clone, PartialEq, DbType)]
    struct MyValue {
        db_id: Option<DbId>,
        name: String,
        age: u64,
    }

    let mut db = TestDb::new();
    let my_value = MyValue {
        db_id: None,
        name: "my name".to_string(),
        age: 20,
    };

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(1)
            .values_uniform(&my_value)
            .query(),
        1,
    );

    let mut db_value: MyValue = db
        .exec_result(
            QueryBuilder::select()
                .values(MyValue::db_keys())
                .ids(1)
                .query(),
        )
        .try_into()
        .unwrap();

    db_value.age = 30;

    db.exec_mut(QueryBuilder::insert().element(&db_value).query(), 2);

    let other: MyValue = db
        .exec_result(
            QueryBuilder::select()
                .values(MyValue::db_keys())
                .ids(1)
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(other, db_value);
}

#[test]
fn insert_multiple_elements() {
    #[derive(Debug, Clone, PartialEq, DbType)]
    struct MyValue {
        db_id: Option<DbId>,
        name: String,
        age: u64,
    }

    let mut db = TestDb::new();
    let my_value = MyValue {
        db_id: None,
        name: "my name".to_string(),
        age: 20,
    };

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(2)
            .values_uniform(&my_value)
            .query(),
        2,
    );

    let mut db_values: Vec<MyValue> = db
        .exec_result(
            QueryBuilder::select()
                .values(MyValue::db_keys())
                .ids([1, 2])
                .query(),
        )
        .try_into()
        .unwrap();

    db_values[0].age = 30;
    db_values[1].age = 40;

    db.exec_mut(QueryBuilder::insert().elements(&db_values).query(), 4);

    let other: Vec<MyValue> = db
        .exec_result(
            QueryBuilder::select()
                .values(MyValue::db_keys())
                .ids([1, 2])
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(other, db_values);
}

#[test]
fn derived_macro_should_not_panic() {
    let mut db = TestDb::new();

    #[derive(Debug, DbType)]
    struct User {
        value: u64,
    }

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([User { value: 0 }.to_db_values()])
            .query(),
        1,
    );

    let user: std::result::Result<User, DbError> = db
        .exec_result(QueryBuilder::search().from(1).query())
        .try_into();

    assert!(user.is_err());
    assert_eq!(user.unwrap_err().description, "Key 'value' not found");
}

#[test]
fn try_from_db_element() {
    let element = DbElement {
        id: DbId(1),
        from: None,
        to: None,
        values: vec![
            ("user_id", 100_u64).into(),
            ("password", "pswd").into(),
            ("status", Status::Active).into(),
        ],
    };

    let user: User = (&element).try_into().unwrap();

    assert_eq!(user.user_id, 100);
    assert_eq!(user.status, Status::Active);
    assert_eq!(user.password, "pswd");
}

#[test]
fn insert_element_alias_update_values() {
    let mut db = TestDb::new();
    let my_value = MyValue {
        db_id: Some("my_alias".into()),
        name: "my name".to_string(),
        age: 20,
    };
    db.exec_mut(
        QueryBuilder::insert().nodes().aliases("my_alias").query(),
        1,
    );
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 2);
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("name", "my name").into(), ("age", 20_u64).into()],
        }],
    );
}

#[test]
fn insert_element_alias_new() {
    let mut db = TestDb::new();
    let my_value = MyValue {
        db_id: Some("my_alias".into()),
        name: "my name".to_string(),
        age: 20,
    };
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 2);
    db.exec_elements(
        QueryBuilder::select().ids("my_alias").query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("name", "my name").into(), ("age", 20_u64).into()],
        }],
    );
}

#[test]
fn insert_element_no_id_new_element() {
    let mut db = TestDb::new();
    let my_value = MyValue {
        db_id: None,
        name: "my name".to_string(),
        age: 20,
    };
    assert_eq!(
        db.exec_mut_result(QueryBuilder::insert().element(&my_value).query()),
        QueryResult {
            result: 2,
            elements: vec![DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![],
            }]
        }
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("name", "my name").into(), ("age", 20_u64).into()],
        }],
    );
}

#[test]
fn insert_element_missing_id() {
    let mut db = TestDb::new();
    let my_value = MyValue {
        db_id: Some(1.into()),
        name: "my name".to_string(),
        age: 20,
    };
    db.exec_mut_error(
        QueryBuilder::insert().element(&my_value).query(),
        "Id '1' not found",
    );
}

#[test]
fn insert_element_bool() {
    let mut db = TestDb::new();
    let mut my_value = MyValueWithBool {
        db_id: None,
        name: "my name".to_string(),
        is_active: true,
        truths: vec![true, false],
    };
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 3);
    let my_value_from_db: MyValueWithBool = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into()
        .unwrap();
    my_value.db_id = Some(QueryId::Id(DbId(1)));
    assert_eq!(my_value, my_value_from_db);
}

#[test]
fn insert_element_to_bool_conversion() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[
                ("name", "my name").into(),
                ("is_active", 50).into(),
                ("truths", vec![1, 0]).into(),
            ]])
            .query(),
        1,
    );
    let my_value_from_db: MyValueWithBool = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into()
        .unwrap();
    let expected = MyValueWithBool {
        db_id: Some(QueryId::Id(DbId(1))),
        name: "my name".to_string(),
        is_active: true,
        truths: vec![true, false],
    };
    assert_eq!(expected, my_value_from_db);
}

#[test]
fn insert_vectorized_custom_types() {
    let mut db = TestDb::new();
    let my_type = MyCustomVec {
        vec: vec![Status::Active, Status::Inactive],
        attributes: vec![
            Attribute {
                name: "name".to_string(),
                value: "value".to_string(),
            },
            Attribute {
                name: "name2".to_string(),
                value: "value2".to_string(),
            },
        ],
    };
    db.exec_mut(QueryBuilder::insert().element(&my_type).query(), 2);
    let my_type_from_db: MyCustomVec = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into()
        .unwrap();
    assert_eq!(my_type, my_type_from_db);
}

#[test]
fn select_user_type() {
    let mut db = TestDb::new();
    let mut my_value = MyValue {
        db_id: None,
        name: "my name".to_string(),
        age: 20,
    };
    let result = db.exec_mut_result(QueryBuilder::insert().element(&my_value).query());
    let my_value_from_db: MyValue = db
        .exec_result(QueryBuilder::select().elements::<MyValue>().ids(1).query())
        .try_into()
        .unwrap();
    my_value.db_id = Some(result.elements[0].id.into());
    assert_eq!(my_value, my_value_from_db);
}

#[test]
fn with_option_some() {
    let mut db = TestDb::new();
    let my_value = WithOption {
        name: "my name".to_string(),
        value: Some(20),
    };
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 2);
    let my_value_from_db: WithOption = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into()
        .unwrap();
    assert_eq!(my_value, my_value_from_db);
}

#[test]
fn with_option_some_wrong_type() {
    let mut db = TestDb::new();
    let my_value = WithOption {
        name: "my name".to_string(),
        value: Some(20),
    };
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 2);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("value", "string").into()]])
            .ids(1)
            .query(),
        1,
    );
    let err: Result<WithOption, DbError> = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into();
    let err_text = err
        .unwrap_err()
        .description
        .split(". (")
        .next()
        .unwrap()
        .to_owned();

    assert_eq!(
        err_text,
        "Failed to convert value of 'value': Type mismatch. Cannot convert 'string' to 'u64'"
    );
}

#[test]
fn with_option_none() {
    let mut db = TestDb::new();
    let my_value = WithOption {
        name: "my name".to_string(),
        value: None,
    };
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 1);
    let my_value_from_db: WithOption = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into()
        .unwrap();
    assert_eq!(my_value, my_value_from_db);
}

#[test]
fn with_option_bad_value() {
    let mut db = TestDb::new();
    let my_value = WithOption {
        name: "my name".to_string(),
        value: Some(20),
    };
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 2);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("name", 100).into()]])
            .ids(1)
            .query(),
        1,
    );
    let err: Result<WithOption, DbError> = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into();
    let err_text = err
        .unwrap_err()
        .description
        .split(". (")
        .next()
        .unwrap()
        .to_owned();

    assert_eq!(
        err_text,
        "Failed to convert value of 'name': Type mismatch. Cannot convert 'i64' to 'string'"
    );
}

#[test]
fn with_option_missing_value() {
    let mut db = TestDb::new();
    let my_value = WithOption {
        name: "my name".to_string(),
        value: Some(20),
    };
    db.exec_mut(QueryBuilder::insert().element(&my_value).query(), 2);
    db.exec_mut(QueryBuilder::remove().values("name").ids(1).query(), -1);
    let err: Result<WithOption, DbError> = db
        .exec_result(QueryBuilder::select().ids(1).query())
        .try_into();

    assert_eq!(err.unwrap_err().description, "Key 'name' not found");
}

#[test]
fn try_from_db_element_bad_conversion() {
    let element = DbElement {
        id: DbId(1),
        from: None,
        to: None,
        values: vec![
            ("user_id", 100_u64).into(),
            ("password", 1_i64).into(),
            ("status", Status::Active).into(),
        ],
    };

    let err: Result<User, DbError> = (&element).try_into();
    let err_text = err
        .unwrap_err()
        .description
        .split(". (")
        .next()
        .unwrap()
        .to_owned();

    assert_eq!(
        err_text,
        "Failed to convert value of 'password': Type mismatch. Cannot convert 'i64' to 'string'"
    );
}

#[test]
fn derived_serialization_struct() {
    #[derive(DbSerialize, Debug, PartialEq)]
    struct S {
        f1: u64,
        f2: u64,
    }

    let s = S { f1: 1, f2: 2 };
    let serialized = s.serialize();
    let deserialized = S::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[test]
fn derived_serialization_tuple() {
    #[derive(DbSerialize, Debug, PartialEq)]
    struct S(u64, u64);

    let s = S(1, 2);
    let serialized = s.serialize();
    let deserialized = S::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[derive(DbSerialize, Debug, PartialEq)]
struct S1 {
    f1: u64,
}

#[derive(DbSerialize, Debug, PartialEq)]
struct S2(S1);

#[derive(DbSerialize, Debug, PartialEq)]
struct S3(S2, S2);

#[derive(DbSerialize, Debug, PartialEq)]
enum MyOtherEnum {
    A,
    B,
}

#[derive(DbSerialize, Debug, PartialEq)]
enum MyE {
    A,
    B(u64),
    C(u64, u64),
    D(S3),
    E(MyOtherEnum),
    F { f1: u64, f2: u64 },
}

#[test]
fn derived_serialization_enum_unit() {
    let s = MyE::A;
    let serialized = s.serialize();
    let deserialized = MyE::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[test]
fn derived_serialization_enum_tuple() {
    let s = MyE::B(1);
    let serialized = s.serialize();
    let deserialized = MyE::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[test]
fn derived_serialization_enum_tuple_multiple() {
    let s = MyE::C(1, 2);
    let serialized = s.serialize();
    let deserialized = MyE::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[test]
fn derived_serialization_enum_nested_struct() {
    let s = MyE::D(S3(S2(S1 { f1: 1 }), S2(S1 { f1: 2 })));
    let serialized = s.serialize();
    let deserialized = MyE::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[test]
fn derived_serialization_enum_nested_enum() {
    let s = MyE::E(MyOtherEnum::A);
    let serialized = s.serialize();
    let deserialized = MyE::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[test]
fn derived_serialization_enum_struct() {
    let s = MyE::F { f1: 1, f2: 2 };
    let serialized = s.serialize();
    let deserialized = MyE::deserialize(&serialized).unwrap();

    assert_eq!(s, deserialized);
}

#[test]
fn derive_serialization_empty_struct() {
    #[derive(DbSerialize, PartialEq, Debug)]

    struct S {}

    let s = S {};
    let serialized = s.serialize();
    let deserialized = S::deserialize(&serialized).unwrap();
    assert_eq!(s, deserialized);
}

#[test]
fn derive_db_type_flatten_nested_struct() {
    #[derive(DbType, PartialEq, Debug)]
    struct Flattened {
        db_id: Option<DbId>,
        category: String,
        #[agdb(flatten)]
        custom: MyCustomVec,
    }

    let mut flattened = Flattened {
        db_id: None,
        category: "test".into(),
        custom: MyCustomVec {
            vec: vec![],
            attributes: vec![],
        },
    };

    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().element(&flattened).query(), 3);

    let keys = db
        .exec_result(QueryBuilder::select().keys().ids(1).query())
        .elements[0]
        .values
        .iter()
        .map(|kv| kv.key.to_string())
        .collect::<Vec<String>>();

    assert_eq!(
        keys,
        vec![
            "category".to_string(),
            "vec".to_string(),
            "attributes".to_string()
        ]
    );

    let retrieved: Flattened = db
        .exec_result(
            QueryBuilder::select()
                .elements::<Flattened>()
                .ids(1)
                .query(),
        )
        .try_into()
        .unwrap();

    flattened.db_id = Some(DbId(1));
    assert_eq!(flattened, retrieved);
}

#[test]
fn derive_db_type_skip_field() {
    #[derive(DbType, PartialEq, Debug)]
    struct Skipped {
        db_id: Option<DbId>,
        category: String,
        #[agdb(skip)]
        custom: MyCustomVec,
    }

    let mut db = TestDb::new();
    let mut skipped = Skipped {
        db_id: None,
        category: "category".to_string(),
        custom: MyCustomVec::default(),
    };
    db.exec_mut(QueryBuilder::insert().element(&skipped).query(), 1);
    let keys = db
        .exec_result(QueryBuilder::select().keys().ids(1).query())
        .elements[0]
        .values
        .iter()
        .map(|kv| kv.key.to_string())
        .collect::<Vec<String>>();

    assert_eq!(keys, vec!["category".to_string()]);

    let retrieved: Skipped = db
        .exec_result(QueryBuilder::select().elements::<Skipped>().ids(1).query())
        .try_into()
        .unwrap();

    skipped.db_id = Some(DbId(1));
    assert_eq!(skipped, retrieved);
}

#[test]
fn derive_db_type_rename_field() {
    #[derive(DbType, PartialEq, Debug)]
    struct Renamed {
        db_id: Option<DbId>,
        #[agdb(rename = "category_name")]
        category: String,
    }

    let mut db = TestDb::new();
    let mut renamed = Renamed {
        db_id: None,
        category: "category".to_string(),
    };
    db.exec_mut(QueryBuilder::insert().element(&renamed).query(), 1);
    let keys = db
        .exec_result(QueryBuilder::select().keys().ids(1).query())
        .elements[0]
        .values
        .iter()
        .map(|kv| kv.key.to_string())
        .collect::<Vec<String>>();

    assert_eq!(keys, vec!["category_name".to_string()]);

    let retrieved: Renamed = db
        .exec_result(QueryBuilder::select().elements::<Renamed>().ids(1).query())
        .try_into()
        .unwrap();

    renamed.db_id = Some(DbId(1));
    assert_eq!(renamed, retrieved);
}

#[test]
fn derive_db_type_skip_generic() {
    #[derive(DbType)]
    struct S {
        db_id: Option<DbId>,
        name: String,
        #[agdb(skip)]
        _generic: std::sync::Arc<u64>,
    }
}

#[test]
fn derive_serialize_vec_t() {
    #[derive(DbSerialize)]
    struct MyVec<T: AgdbSerialize> {
        values: Vec<T>,
    }
    let _ = MyVec {
        values: vec![1_u64, 2_u64],
    };
}

#[test]
fn derive_db_value_vec_t() {
    #[derive(DbValue, DbSerialize)]
    struct MyVec<T: AgdbSerialize> {
        values: Vec<T>,
    }
}

#[test]
fn derive_db_type_db_id_no_option() {
    #[derive(DbType, PartialEq)]
    struct S {
        db_id: DbId,
        name: String,
    }

    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .element(&S {
                db_id: DbId::default(),
                name: "name".to_string(),
            })
            .query(),
        1,
    );
    let s: S = db
        .exec_result(QueryBuilder::select().elements::<S>().ids(1).query())
        .try_into()
        .unwrap();
    assert_eq!(s.db_id, DbId(1));
    assert_eq!(s.name, "name");
}

#[test]
fn derive_db_element() {
    #[derive(DbElement)]
    struct Type1 {
        db_id: DbId,
        name: String,
    }

    #[derive(DbElement)]
    struct Type2 {
        db_id: DbId,
        name: String,
    }

    let mut db = TestDb::new();
    let root_id = db
        .exec_mut_result(QueryBuilder::insert().nodes().aliases("root").query())
        .elements[0]
        .id;
    let ty1 = db
        .exec_mut_result(
            QueryBuilder::insert()
                .element(&Type1 {
                    db_id: DbId::default(),
                    name: "type1".to_string(),
                })
                .query(),
        )
        .elements[0]
        .id;
    let ty2 = db
        .exec_mut_result(
            QueryBuilder::insert()
                .element(&Type2 {
                    db_id: DbId::default(),
                    name: "type2".to_string(),
                })
                .query(),
        )
        .elements[0]
        .id;
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(root_id)
            .to([ty1, ty2])
            .query(),
        2,
    );

    let ty1_result: Vec<Type1> = db
        .exec_result(
            QueryBuilder::select()
                .elements::<Type1>()
                .search()
                .from("root")
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(ty1_result.len(), 1);
    assert_eq!(ty1_result[0].name, "type1");
}

#[test]
fn insert_element_by_value() {
    #[derive(DbElement)]
    struct Type1 {
        name: String,
    }

    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .element(Type1 {
                name: "test".to_string(),
            })
            .query(),
        2,
    );
}
