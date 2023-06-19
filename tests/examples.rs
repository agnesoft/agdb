use agdb::Comparison;
use agdb::Db;
use agdb::QueryBuilder;

#[rustfmt::skip]
#[test]
fn quickstart() {
    let _ = std::fs::remove_file("db_file.agdb");
    let mut db = Db::new("db_file.agdb").unwrap();

    //create a nodes with data
    db.exec_mut(&QueryBuilder::insert().nodes().aliases(&["users".into()]).query()).unwrap();
    let users = db.exec_mut(&QueryBuilder::insert().nodes().values(&[
        &[("id", 1).into(), ("username", "user_1").into()],
        &[("id", 2).into(), ("username", "user_2").into()],
        &[("id", 3).into(), ("username", "user_3").into()]]
    ).query()).unwrap();

    //connect nodes
    db.exec_mut(&QueryBuilder::insert().edges().from(&["users".into()]).to(&users.ids()).query()).unwrap();

    //select nodes
    let user_elements = db.exec(&QueryBuilder::select().ids(&users.ids()).query()).unwrap();

    for element in user_elements.elements {
        println!("{:?}: {:?}", element.id, element.values);
    }

    // DbId(2): [DbKeyValue { key: String("id"), value: Int(1) }, DbKeyValue { key: String("username"), value: String("user_1") }]
    // DbId(3): [DbKeyValue { key: String("id"), value: Int(2) }, DbKeyValue { key: String("username"), value: String("user_2") }]
    // DbId(4): [DbKeyValue { key: String("id"), value: Int(3) }, DbKeyValue { key: String("username"), value: String("user_3") }]

    //search with conditions
    let user_id = db.exec(&QueryBuilder::search().from("users").where_().key("username").value(Comparison::Equal("user_2".into())).query()).unwrap();

    println!("{:?}", user_id.elements);
    //[DbElement { id: DbId(3), values: [] }]
    let _ = std::fs::remove_file("db_file.agdb");
}
