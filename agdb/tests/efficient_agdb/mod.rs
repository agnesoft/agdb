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
use agdb_derive::UserValue;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(UserValue)]
struct User {
    username: String,
    email: String,
    password: String,
}

#[derive(UserValue)]
struct Post {
    db_id: Option<DbId>,
    title: String,
    body: String,
}

#[derive(UserValue)]
struct PostLiked {
    db_id: Option<DbId>,
    title: String,
    body: String,
    likes: i64,
}

#[derive(UserValue)]
struct Comment {
    body: String,
}

fn create_db() -> Result<Arc<RwLock<Db>>, QueryError> {
    let db = Arc::new(RwLock::new(Db::new("social.agdb")?));
    db.write()?.transaction_mut(|t| -> Result<(), QueryError> {
        t.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(["root", "users", "posts"])
                .query(),
        )?;
        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from("root")
                .to(["users", "posts"])
                .query(),
        )?;
        Ok(())
    })?;

    Ok(db)
}

fn register_user(db: &mut Db, user: &User) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
        if t.exec(
            QueryBuilder::search()
                .from("users")
                .where_()
                .key("username")
                .value(Equal(user.username.clone().into()))
                .query(),
        )?
        .result
            != 0
        {
            return Err(QueryError::from(format!(
                "User {} already exists.",
                user.username
            )));
        }

        let user = t
            .exec_mut(QueryBuilder::insert().element(user).query())?
            .elements[0]
            .id;

        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from("users")
                .to(user)
                .query(),
        )?;

        Ok(user)
    })
}

fn create_post(db: &mut Db, user: DbId, post: &Post) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
        let post = t
            .exec_mut(QueryBuilder::insert().element(post).query())?
            .elements[0]
            .id;

        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from([QueryId::from("posts"), user.into()])
                .to(post)
                .values([vec![], vec![("authored", 1_u64).into()]])
                .query(),
        )?;

        Ok(post)
    })
}

fn create_comment(
    db: &mut Db,
    user: DbId,
    parent: DbId,
    comment: &Comment,
) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
        let comment = t
            .exec_mut(QueryBuilder::insert().element(comment).query())?
            .elements[0]
            .id;

        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from([parent, user])
                .to(comment)
                .values([vec![], vec![("commented", 1_u64).into()]])
                .query(),
        )?;

        Ok(comment)
    })
}

fn like(db: &mut Db, user: DbId, id: DbId) -> Result<(), QueryError> {
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(user)
            .to(id)
            .values_uniform([("liked", 1).into()])
            .query(),
    )?;
    Ok(())
}

fn remove_like(db: &mut Db, user: DbId, id: DbId) -> Result<(), QueryError> {
    db.transaction_mut(|t| -> Result<(), QueryError> {
        t.exec_mut(
            QueryBuilder::remove()
                .search()
                .from(user)
                .to(id)
                .where_()
                .keys("liked")
                .query(),
        )?;
        Ok(())
    })
}

fn login(db: &Db, username: &str, password: &str) -> Result<DbId, QueryError> {
    let result = db
        .exec(
            QueryBuilder::select()
                .values("password")
                .search()
                .depth_first()
                .from("users")
                .limit(1)
                .where_()
                .distance(CountComparison::Equal(2))
                .and()
                .key("username")
                .value(Equal(username.into()))
                .query(),
        )?
        .elements;

    let user = result
        .first()
        .ok_or(QueryError::from(format!("Username '{username}' not found")))?;

    let pswd = user.values[0].value.to_string();

    if password != pswd {
        return Err(QueryError::from("Password is incorrect"));
    }

    Ok(user.id)
}

fn user_posts_ids(db: &Db, user: DbId) -> Result<Vec<DbId>, QueryError> {
    Ok(db
        .exec(
            QueryBuilder::search()
                .from(user)
                .where_()
                .distance(CountComparison::Equal(2))
                .and()
                .beyond()
                .where_()
                .keys("authored")
                .or()
                .node()
                .query(),
        )?
        .ids())
}

fn post_titles(db: &Db, ids: Vec<DbId>) -> Result<Vec<String>, QueryError> {
    Ok(db
        .exec(QueryBuilder::select().values("title").ids(ids).query())?
        .elements
        .into_iter()
        .map(|post| post.values[0].value.to_string())
        .collect())
}

fn posts(db: &Db, offset: u64, limit: u64) -> Result<Vec<Post>, QueryError> {
    Ok(db
        .exec(
            QueryBuilder::select()
                .elements::<Post>()
                .search()
                .from("posts")
                .offset(offset)
                .limit(limit)
                .where_()
                .distance(CountComparison::Equal(2))
                .query(),
        )?
        .try_into()?)
}

fn comments(db: &Db, id: DbId) -> Result<Vec<Comment>, QueryError> {
    Ok(db
        .exec(
            QueryBuilder::select()
                .elements::<Comment>()
                .search()
                .depth_first()
                .from(id)
                .where_()
                .node()
                .and()
                .distance(CountComparison::GreaterThan(1))
                .query(),
        )?
        .try_into()?)
}

fn add_likes_to_posts(db: &mut Db) -> Result<(), QueryError> {
    db.transaction_mut(|t| -> Result<(), QueryError> {
        let posts = t.exec(
            QueryBuilder::search()
                .from("posts")
                .where_()
                .distance(CountComparison::Equal(2))
                .query(),
        )?;
        let mut likes = Vec::<Vec<DbKeyValue>>::new();

        for post in posts.ids() {
            let post_likes = t
                .exec(
                    QueryBuilder::search()
                        .to(post)
                        .where_()
                        .distance(CountComparison::Equal(1))
                        .and()
                        .keys("liked")
                        .query(),
                )?
                .result;
            likes.push(vec![("likes", post_likes).into()]);
        }

        t.exec_mut(QueryBuilder::insert().values(likes).ids(posts).query())?;
        Ok(())
    })
}

fn liked_posts(db: &Db, offset: u64, limit: u64) -> Result<Vec<PostLiked>, QueryError> {
    Ok(db
        .exec(
            QueryBuilder::select()
                .elements::<PostLiked>()
                .search()
                .from("posts")
                .order_by([DbKeyOrder::Desc("likes".into())])
                .offset(offset)
                .limit(limit)
                .where_()
                .distance(CountComparison::Equal(2))
                .query(),
        )?
        .try_into()?)
}

fn mark_top_level_comments(db: &mut Db) -> Result<(), QueryError> {
    db.exec_mut(
        QueryBuilder::insert()
            .values_uniform([("level", 1).into()])
            .search()
            .from("posts")
            .where_()
            .distance(CountComparison::Equal(4))
            .query(),
    )?;
    Ok(())
}

#[test]
fn efficient_agdb() -> Result<(), QueryError> {
    let _test_file = TestFile::from("social.agdb");
    let db = create_db()?;

    let user = User {
        username: "john_doe".to_string(),
        email: "john@doe.com".to_string(),
        password: "password123".to_string(),
    };
    register_user(db.write()?.deref_mut(), &user)?;
    let user_id = login(db.read()?.deref(), "john_doe", "password123")?;

    let post = Post {
        db_id: None,
        title: "Awesome".to_string(),
        body: "http://pictures.com/awesome.png".to_string(),
    };
    let post_id = create_post(db.write()?.deref_mut(), user_id, &post)?;

    let comment = Comment {
        body: "This is truly awesome".to_string(),
    };
    let comment_id = create_comment(db.write()?.deref_mut(), user_id, post_id, &comment)?;

    let reply_comment = Comment {
        body: "Indeed it is".to_string(),
    };
    let reply_comment_id =
        create_comment(db.write()?.deref_mut(), user_id, comment_id, &reply_comment)?;

    like(db.write()?.deref_mut(), user_id, post_id)?;
    like(db.write()?.deref_mut(), user_id, comment_id)?;
    like(db.write()?.deref_mut(), user_id, reply_comment_id)?;
    remove_like(db.write()?.deref_mut(), user_id, comment_id)?;

    let posts = posts(db.read()?.deref(), 0, 10)?;
    assert_eq!(posts.len(), 1);

    let user_posts = user_posts_ids(db.read()?.deref(), user_id)?;
    assert_eq!(user_posts.len(), 1);

    let user_post_titles = post_titles(db.read()?.deref(), user_posts)?;
    assert_eq!(user_post_titles, vec!["Awesome"]);

    let comments = comments(db.read()?.deref(), post_id)?;
    assert_eq!(comments.len(), 2);

    add_likes_to_posts(db.write()?.deref_mut())?;
    mark_top_level_comments(db.write()?.deref_mut())?;

    let posts = liked_posts(db.read()?.deref(), 0, 10)?;
    assert_eq!(posts.len(), 1);

    Ok(())
}
