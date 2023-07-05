mod test_db;

use crate::test_db::TestFile;
use agdb::Comparison::Equal;
use agdb::CountComparison;
use agdb::CountComparison::LessThanOrEqual;
use agdb::Db;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::DbKeyValue;
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
    let _test_file = TestFile::from("myplace.agdb");

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
    let body = format!("https://photos.myplace.net/{}/car.png", user.0);
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
                    .from(vec![QueryId::from("posts"), user.into()])
                    .to(post)
                    .values(vec![vec![], vec![("authored", timestamp).into()]])
                    .query(),
            )?;
            Ok(post)
        })?;

    let body = "I have this car since 2008 only in red. It's a great car!";
    let timestamp = 456;
    let top_comment = db
        .write()?
        .transaction_mut(|t| -> Result<DbId, QueryError> {
            let comment = t
                .exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .values(vec![vec![("body", body).into()]])
                        .query(),
                )?
                .elements[0]
                .id;
            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(vec![post, user])
                    .to(comment)
                    .values(vec![vec![], vec![("commented", timestamp).into()]])
                    .query(),
            )?;
            Ok(comment)
        })?;

    let body = "They stopped making them just a year later in 2009 and the next generation flopped so they don't make them anymore. It's a shame, it really was a good car.";
    let timestamp = 456;
    let _reply_comment = db
        .write()?
        .transaction_mut(|t| -> Result<DbId, QueryError> {
            let comment = t
                .exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .values(vec![vec![("body", body).into()]])
                        .query(),
                )?
                .elements[0]
                .id;
            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(vec![top_comment, user])
                    .to(comment)
                    .values(vec![vec![], vec![("commented", timestamp).into()]])
                    .query(),
            )?;
            Ok(comment)
        })?;

    db.write()?.exec_mut(
        &QueryBuilder::insert()
            .edges()
            .from(user)
            .to(vec![post, top_comment])
            .values_uniform(vec![("liked", 1).into()])
            .query(),
    )?;

    let user = db
        .read()?
        .exec(
            &QueryBuilder::search()
                .depth_first()
                .from("users")
                .limit(1)
                .where_()
                .distance(LessThanOrEqual(2))
                .and()
                .key("username")
                .value(Equal(username.into()))
                .and()
                .key("password")
                .value(Equal(password.into()))
                .query(),
        )?
        .elements
        .get(0)
        .ok_or(QueryError::from("Username or password are incorrect"))?
        .id;

    let _user_posts = db.read()?.exec(
        &QueryBuilder::search()
            .from(user)
            .where_()
            .distance(CountComparison::Equal(2))
            .and()
            .beyond()
            .where_()
            .node()
            .or()
            .keys(vec!["authored".into()])
            .query(),
    )?;

    let posts = db.read()?.exec(
        &QueryBuilder::select()
            .search(
                QueryBuilder::search()
                    .from("posts")
                    .order_by(vec![DbKeyOrder::Desc("likes".into())])
                    .offset(0)
                    .limit(10)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .query(),
            )
            .query(),
    )?;

    let _comments = db.read()?.exec(
        &QueryBuilder::select()
            .search(
                QueryBuilder::search()
                    .depth_first()
                    .from(posts.elements[0].id)
                    .query(),
            )
            .query(),
    )?;

    db.write()?.transaction_mut(|t| -> Result<(), QueryError> {
        let posts = t.exec(
            &QueryBuilder::search()
                .from("posts")
                .where_()
                .distance(CountComparison::Equal(2))
                .query(),
        )?;
        let mut likes = Vec::<Vec<DbKeyValue>>::new();

        for post in posts.ids() {
            let post_likes = t
                .exec(
                    &QueryBuilder::search()
                        .to(post)
                        .where_()
                        .distance(CountComparison::Equal(1))
                        .and()
                        .keys(vec!["liked".into()])
                        .query(),
                )?
                .result;
            likes.push(vec![("likes", post_likes).into()]);
        }

        t.exec_mut(&QueryBuilder::insert().values(likes).ids(posts).query())?;
        Ok(())
    })?;

    db.write()?.exec_mut(
        &QueryBuilder::insert()
            .values_uniform(vec![("level", 1).into()])
            .search(
                QueryBuilder::search()
                    .from("posts")
                    .where_()
                    .distance(CountComparison::Equal(4))
                    .query(),
            )
            .query(),
    )?;

    Ok(())
}
