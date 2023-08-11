use agdb::DbError;
use agdb::DbKey;
use agdb::DbKeyValue;
use agdb::DbUserValue;
use agdb::DbValue;
use agdb_derive::UserValue;

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

    let values: Vec<DbKeyValue> = vec![
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
    ];

    assert_eq!(my_data.db_keys(), keys);
    assert_eq!(&my_data.db_values(), &values);
    assert_eq!(MyData::from_db_values(&values).unwrap(), my_data);
}
