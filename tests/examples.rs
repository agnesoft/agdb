mod test_db;

use crate::test_db::TestFile;
use agdb::Comparison;
use agdb::Db;
use agdb::QueryBuilder;

#[rustfmt::skip]
#[test]
fn quickstart() {
    let _test_file = TestFile::from("db_file.agdb");

    let mut db = Db::new("db_file.agdb").unwrap();
    let insert_users_root = QueryBuilder::insert().nodes().aliases(&["users".into()]).query();
    db.exec_mut(&insert_users_root).unwrap();

    let insert_users = QueryBuilder::insert().nodes().values(&[
            &[("id", 1).into(), ("username", "user_1").into()],
            &[("id", 2).into(), ("username", "user_2").into()],
            &[("id", 3).into(), ("username", "user_3").into()],
        ]).query();
    let users = db.exec_mut(&insert_users).unwrap();

    let insert_edges = QueryBuilder::insert().edges().from(&["users".into()]).to(&users.ids()).query();
    db.exec_mut(&insert_edges).unwrap();

    let select_users = QueryBuilder::select().ids(&users.ids()).query();
    let user_elements = db.exec(&select_users).unwrap();

    println!("{:?}", user_elements);
    // QueryResult {
    //   result: 3,
    //   elements: [
    //     DbElement { id: DbId(2), values: [DbKeyValue { key: String("id"), value: Int(1) }, DbKeyValue { key: String("username"), value: String("user_1") }] },
    //     DbElement { id: DbId(3), values: [DbKeyValue { key: String("id"), value: Int(2) }, DbKeyValue { key: String("username"), value: String("user_2") }] },
    //     DbElement { id: DbId(4), values: [DbKeyValue { key: String("id"), value: Int(3) }, DbKeyValue { key: String("username"), value: String("user_3") }] }
    // ] }

    let search_for_user = QueryBuilder::search().from("users").where_().key("username").value(Comparison::Equal("user_2".into())).query();
    let select_user = QueryBuilder::select().search(search_for_user).query();
    let user_id = db.exec(&select_user).unwrap();

    println!("{:?}", user_id);
    // QueryResult {
    //   result: 1,
    //   elements: [
    //     DbElement { id: DbId(3), values: [DbKeyValue { key: String("id"), value: Int(2) }, DbKeyValue { key: String("username"), value: String("user_2") }] }
    //   ] }
}
