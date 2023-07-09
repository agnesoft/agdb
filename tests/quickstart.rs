mod test_db;

use crate::test_db::TestFile;
use agdb::Comparison::Equal;
use agdb::Db;
use agdb::QueryBuilder;
use agdb::QueryError;

#[test]
fn quickstart() -> Result<(), QueryError> {
    let _test_file = TestFile::from("db_file.agdb");

    let mut db = Db::new("db_file.agdb")?;

    db.exec_mut(&QueryBuilder::insert().nodes().aliases("users").query())?;

    let users = db.exec_mut(
        &QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("id", 1).into(), ("username", "user_1").into()],
                vec![("id", 2).into(), ("username", "user_2").into()],
                vec![("id", 3).into(), ("username", "user_3").into()],
            ])
            .query(),
    )?;

    db.exec_mut(
        &QueryBuilder::insert()
            .edges()
            .from("users")
            .to(&users)
            .query(),
    )?;

    let user_elements = db.exec(&QueryBuilder::select().ids(users).query())?;
    println!("{:?}", user_elements);
    // QueryResult {
    //   result: 3,
    //   elements: [
    //     DbElement { id: DbId(2), values: [DbKeyValue { key: String("id"), value: Int(1) }, DbKeyValue { key: String("username"), value: String("user_1") }] },
    //     DbElement { id: DbId(3), values: [DbKeyValue { key: String("id"), value: Int(2) }, DbKeyValue { key: String("username"), value: String("user_2") }] },
    //     DbElement { id: DbId(4), values: [DbKeyValue { key: String("id"), value: Int(3) }, DbKeyValue { key: String("username"), value: String("user_3") }] }
    // ] }

    let user_id = db.exec(
        &QueryBuilder::select()
            .ids(
                QueryBuilder::search()
                    .from("users")
                    .where_()
                    .key("username")
                    .value(Equal("user_2".into()))
                    .query(),
            )
            .query(),
    )?;

    println!("{:?}", user_id);
    // QueryResult {
    //   result: 1,
    //   elements: [
    //     DbElement { id: DbId(3), values: [DbKeyValue { key: String("id"), value: Int(2) }, DbKeyValue { key: String("username"), value: String("user_2") }] }
    //   ] }

    Ok(())
}
