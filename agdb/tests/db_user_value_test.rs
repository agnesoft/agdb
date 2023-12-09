mod test_db;

use agdb::DbElement;
use agdb::DbError;
use agdb::DbId;
use agdb::DbKey;
use agdb::DbUserValue;
use agdb::DbValue;
use agdb::QueryBuilder;
use agdb::UserValue;
use test_db::TestDb;

#[derive(Default, Debug, Clone, PartialEq)]
enum Status {
    Active,
    #[default]
    Inactive,
}

#[derive(UserValue)]
struct User {
    user_id: u64,
    password: String,
    status: Status,
}

impl From<Status> for DbValue {
    fn from(value: Status) -> Self {
        match value {
            Status::Active => DbValue::I64(1),
            Status::Inactive => DbValue::I64(0),
        }
    }
}

impl TryFrom<DbValue> for Status {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        if value.to_u64()? == 0 {
            Ok(Status::Inactive)
        } else {
            Ok(Status::Active)
        }
    }
}

#[test]
fn db_user_value() {
    #[derive(Default, Debug, PartialEq, UserValue)]
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
    let keys: Vec<DbKey> = vec![
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
    #[derive(UserValue)]
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
            values: vec![("name", "my name").into(), ("age", 20_u64).into()],
        }],
    );
}

#[test]
fn insert_node_values_uniform_custom() {
    #[derive(UserValue)]
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
        QueryBuilder::select().ids(vec![1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                values: vec![("name", "my name").into(), ("age", 20_u64).into()],
            },
            DbElement {
                id: DbId(2),
                values: vec![("name", "my name").into(), ("age", 20_u64).into()],
            },
        ],
    );
}

#[test]
fn select_custom_value_keys() {
    #[derive(Debug, Clone, PartialEq, UserValue)]
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
                .ids(vec![1, 2])
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(db_values, vec![my_value.clone(), my_value]);
}

#[test]
fn select_custom_value_with_id() {
    #[derive(Debug, Clone, PartialEq, UserValue)]
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
                .ids(vec![1, 2])
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
    #[derive(Debug, Clone, PartialEq, UserValue)]
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
    #[derive(Debug, Clone, PartialEq, UserValue)]
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
                .ids(vec![1, 2])
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
                .ids(vec![1, 2])
                .query(),
        )
        .try_into()
        .unwrap();

    assert_eq!(other, db_values);
}

#[test]
fn derived_macro_should_not_panic() {
    let mut db = TestDb::new();

    #[derive(Debug, UserValue)]
    struct User {
        value: u64,
    }

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![User { value: 0 }.to_db_values()])
            .query(),
        1,
    );

    let user: Result<User, DbError> = db
        .exec_result(QueryBuilder::search().from(1).query())
        .try_into();

    assert!(user.is_err());
    assert_eq!(user.unwrap_err().description, "Not enough keys");
}

#[test]
fn try_from_db_element() {
    let element = DbElement {
        id: DbId(1),
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
