mod test_db;

use crate::test_db::TestFile;
use agdb::Comparison::Equal;
use agdb::CountComparison;
use agdb::Db;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::DbKeyValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryId;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::RwLock;

fn create_db() -> Result<Arc<RwLock<Db>>, QueryError> {
    let db = Arc::new(RwLock::new(Db::new("database.agdb")?));
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

    Ok(db)
}

fn register_user(
    db: &mut Db,
    username: &str,
    email: &str,
    password: &str,
) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
        if t.exec(
            &QueryBuilder::search()
                .from("users")
                .where_()
                .key("username")
                .value(Equal(username.into()))
                .query(),
        )?
        .result
            != 0
        {
            return Err(QueryError::from(format!("User {username} already exists.")));
        }

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
    })
}

fn create_post(db: &mut Db, user: DbId, title: &str, body: &str) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
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
                .values(vec![vec![], vec![("authored", 1).into()]])
                .query(),
        )?;
        Ok(post)
    })
}

fn create_comment(db: &mut Db, user: DbId, parent: DbId, body: &str) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
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
                .from(vec![parent, user])
                .to(comment)
                .values(vec![vec![], vec![("commented", 1).into()]])
                .query(),
        )?;
        Ok(comment)
    })
}

fn like(db: &mut Db, user: DbId, id: DbId) -> Result<(), QueryError> {
    db.exec_mut(
        &QueryBuilder::insert()
            .edges()
            .from(user)
            .to(id)
            .values_uniform(vec![("liked", 1).into()])
            .query(),
    )?;
    Ok(())
}

fn remove_like(db: &mut Db, user: DbId, id: DbId) -> Result<(), QueryError> {
    db.transaction_mut(|t| -> Result<(), QueryError> {
        t.exec_mut(
            &QueryBuilder::remove()
                .ids(
                    QueryBuilder::search()
                        .from(user)
                        .to(id)
                        .where_()
                        .keys(vec!["liked".into()])
                        .query(),
                )
                .query(),
        )?;
        Ok(())
    })
}

fn login(db: &Db, username: &str, password: &str) -> Result<DbId, QueryError> {
    let result = db
        .exec(
            &QueryBuilder::select()
                .values(vec!["password".into()])
                .ids(
                    QueryBuilder::search()
                        .depth_first()
                        .from("users")
                        .limit(1)
                        .where_()
                        .distance(CountComparison::Equal(2))
                        .and()
                        .key("username")
                        .value(Equal(username.into()))
                        .query(),
                )
                .query(),
        )?
        .elements;

    let user = result
        .get(0)
        .ok_or(QueryError::from(format!("Username '{username}' not found")))?;

    let pswd = user.values[0].value.to_string();

    if password != pswd {
        return Err(QueryError::from("Password is incorrect"));
    }

    Ok(user.id)
}

fn user_posts(db: &Db, user: DbId) -> Result<Vec<DbId>, QueryError> {
    Ok(db
        .exec(
            &QueryBuilder::search()
                .from(user)
                .where_()
                .distance(CountComparison::Equal(2))
                .and()
                .beyond()
                .where_()
                .keys(vec!["authored".into()])
                .or()
                .node()
                .query(),
        )?
        .ids())
}

fn post_titles(db: &Db, ids: Vec<DbId>) -> Result<Vec<String>, QueryError> {
    Ok(db
        .exec(
            &QueryBuilder::select()
                .values(vec!["title".into()])
                .ids(ids)
                .query(),
        )?
        .elements
        .into_iter()
        .map(|post| post.values[0].value.to_string())
        .collect())
}

fn posts(db: &Db, offset: u64, limit: u64) -> Result<Vec<DbId>, QueryError> {
    Ok(db
        .exec(
            &QueryBuilder::select()
                .ids(
                    QueryBuilder::search()
                        .from("posts")
                        .offset(offset)
                        .limit(limit)
                        .where_()
                        .distance(CountComparison::Equal(2))
                        .query(),
                )
                .query(),
        )?
        .ids())
}

fn liked_posts(db: &Db, offset: u64, limit: u64) -> Result<Vec<DbId>, QueryError> {
    Ok(db
        .exec(
            &QueryBuilder::select()
                .ids(
                    QueryBuilder::search()
                        .from("posts")
                        .order_by(vec![DbKeyOrder::Desc("likes".into())])
                        .offset(offset)
                        .limit(limit)
                        .where_()
                        .distance(CountComparison::Equal(2))
                        .query(),
                )
                .query(),
        )?
        .ids())
}

fn comments(db: &Db, id: DbId) -> Result<Vec<String>, QueryError> {
    Ok(db
        .exec(
            &QueryBuilder::select()
                .values(vec!["body".into()])
                .ids(
                    QueryBuilder::search()
                        .depth_first()
                        .from(id)
                        .where_()
                        .node()
                        .and()
                        .distance(CountComparison::GreaterThan(1))
                        .query(),
                )
                .query(),
        )?
        .elements
        .into_iter()
        .map(|c| c.values[0].value.to_string())
        .collect())
}

fn add_likes_to_posts(db: &mut Db) -> Result<(), QueryError> {
    db.transaction_mut(|t| -> Result<(), QueryError> {
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
    })
}

fn mark_top_level_comments(db: &mut Db) -> Result<(), QueryError> {
    db.exec_mut(
        &QueryBuilder::insert()
            .values_uniform(vec![("level", 1).into()])
            .ids(
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

#[test]
fn efficient_agdb() -> Result<(), QueryError> {
    let _test_file = TestFile::from("database.agdb");
    let db = create_db()?;
    register_user(
        db.write()?.deref_mut(),
        "john_doe",
        "john@doe.com",
        "password123",
    )?;
    let user = login(db.read()?.deref(), "john_doe", "password123")?;
    let post = create_post(
        db.write()?.deref_mut(),
        user,
        "Awesome car",
        "http://pictures.com/awesome_car.png",
    )?;
    let comment = create_comment(db.write()?.deref_mut(), user, post, "This is truly awesome")?;
    let reply = create_comment(db.write()?.deref_mut(), user, comment, "Indeed it is")?;
    like(db.write()?.deref_mut(), user, post)?;
    like(db.write()?.deref_mut(), user, comment)?;
    like(db.write()?.deref_mut(), user, reply)?;
    remove_like(db.write()?.deref_mut(), user, comment)?;

    let posts = posts(db.read()?.deref(), 0, 10)?;
    assert_eq!(posts.len(), 1);

    let posts = user_posts(db.read()?.deref(), user)?;
    assert_eq!(posts.len(), 1);

    let titles = post_titles(db.read()?.deref(), posts)?;
    assert_eq!(titles, vec!["Awesome car"]);

    let comments = comments(db.read()?.deref(), post)?;
    assert_eq!(comments.len(), 2);

    add_likes_to_posts(db.write()?.deref_mut())?;
    mark_top_level_comments(db.write()?.deref_mut())?;

    let posts = liked_posts(db.read()?.deref(), 0, 10)?;
    assert_eq!(posts.len(), 1);

    Ok(())
}
