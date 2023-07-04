mod test_db;

use crate::test_db::TestFile;
use agdb::Comparison::Equal;
use agdb::Db;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryId;
use std::sync::Arc;
use std::sync::RwLock;

#[test]
fn quickstart() -> Result<(), QueryError> {
    let _test_file = TestFile::from("db_file.agdb");

    let mut db = Db::new("db_file.agdb")?;
    let insert_users_root = QueryBuilder::insert().nodes().aliases("users").query();
    db.exec_mut(&insert_users_root)?;

    let user_values = vec![
        vec![("id", 1).into(), ("username", "user_1").into()],
        vec![("id", 2).into(), ("username", "user_2").into()],
        vec![("id", 3).into(), ("username", "user_3").into()],
    ];
    let users = db.exec_mut(&QueryBuilder::insert().nodes().values(user_values).query())?;

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
            .search(
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

#[test]
fn guide() -> Result<(), QueryError> {
    let db = Arc::new(RwLock::new(Db::new("myplace.agdb")?));
    db.write()?.transaction_mut(|t| -> Result<(), QueryError> {
        t.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(vec!["root", "users", "posts"])
                .query(),
        )?;
        t.exec_mut(
            &QueryBuilder::insert()
                .edges()
                .from("root")
                .to(vec!["users", "posts"])
                .query(),
        )?;
        Ok(())
    })?;

    let username = "luckyjoe";
    let email = "lucky.joe@internet.net";
    let password = "mypassword123";

    let user = db
        .write()?
        .transaction_mut(|t| -> Result<DbId, QueryError> {
            let user = t
                .exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .values(vec![vec![
                            ("username", username).into(),
                            ("email", email).into(),
                            ("password", password).into(),
                        ]])
                        .query(),
                )?
                .elements[0]
                .id;
            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from("users")
                    .to(user)
                    .query(),
            )?;
            Ok(user)
        })?;

    let title = "My awesome car";
    let body = format!("https://photos.myplace.net/{}/car.jpeg", user.0);
    let timestamp = 123;

    let post = db
        .write()?
        .transaction_mut(|t| -> Result<DbId, QueryError> {
            let post = t
                .exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .values(vec![vec![
                            ("title", title).into(),
                            ("body", body.clone()).into(),
                        ]])
                        .query(),
                )?
                .elements[0]
                .id;
            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(vec![QueryId::from("users"), user.into()])
                    .to(post)
                    .values(vec![vec![], vec![("authored", timestamp).into()]])
                    .query(),
            )?;
            Ok(post)
        })?;

    Ok(())
}
