use agdb::DbId;
use agdb::DbType;
use agdb::Query;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::QueryMut;

#[derive(DbType)]
pub(crate) struct BenchUser {
    pub(crate) name: String,
    pub(crate) email: String,
}

#[derive(DbType)]
pub(crate) struct BenchPost {
    pub(crate) title: String,
    pub(crate) body: String,
}

#[derive(DbType)]
pub(crate) struct BenchComment {
    pub(crate) body: String,
}

pub(crate) fn insert_user_query(
    name: String,
    email: String,
) -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert()
        .nodes()
        .values(BenchUser { name, email })
        .query()
}

pub(crate) fn insert_users_alias_query() -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert().nodes().aliases("users").query()
}

pub(crate) fn insert_posts_alias_query() -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert().nodes().aliases("posts").query()
}

pub(crate) fn bootstrap_alias_queries() -> Vec<agdb::QueryType> {
    vec![
        insert_users_alias_query().into(),
        insert_posts_alias_query().into(),
    ]
}

pub(crate) fn insert_user_edges_query(
    user_ids: Vec<DbId>,
) -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert()
        .edges()
        .from("users")
        .to(user_ids)
        .query()
}

pub(crate) fn insert_post_query(title: &str, body: &str) -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert()
        .nodes()
        .values(BenchPost {
            title: title.to_string(),
            body: body.to_string(),
        })
        .query()
}

pub(crate) fn insert_comment_query(body: &str) -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert()
        .nodes()
        .values(BenchComment {
            body: body.to_string(),
        })
        .query()
}

pub(crate) fn insert_post_authored_edge_query(
    user_id: DbId,
    post_id: QueryId,
) -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert()
        .edges()
        .from([QueryId::from("posts"), user_id.into()])
        .to(post_id)
        .values([[].as_slice(), &[("authored", 1).into()]])
        .query()
}

pub(crate) fn insert_comment_edge_query(
    post_id: DbId,
    user_id: DbId,
    comment_id: QueryId,
) -> impl QueryMut + Into<agdb::QueryType> {
    QueryBuilder::insert()
        .edges()
        .from([post_id, user_id])
        .to(comment_id)
        .values([[].as_slice(), &[("commented", 1).into()]])
        .query()
}

pub(crate) fn select_post_writer_users_query(limit: u64) -> impl Query + Into<agdb::QueryType> {
    QueryBuilder::search()
        .from("users")
        .limit(limit)
        .where_()
        .neighbor()
        .query()
}

pub(crate) fn select_comment_writer_users_query(
    offset: u64,
    limit: u64,
) -> impl Query + Into<agdb::QueryType> {
    QueryBuilder::search()
        .from("users")
        .offset(offset)
        .limit(limit)
        .where_()
        .neighbor()
        .query()
}

pub(crate) fn select_posts_query(limit: u64) -> impl Query + Into<agdb::QueryType> {
    QueryBuilder::select()
        .ids(
            QueryBuilder::search()
                .from("posts")
                .limit(limit)
                .where_()
                .neighbor()
                .query(),
        )
        .query()
}

pub(crate) fn select_comments_query(
    post_id: DbId,
    limit: u64,
) -> impl Query + Into<agdb::QueryType> {
    QueryBuilder::select()
        .ids(
            QueryBuilder::search()
                .from(post_id)
                .limit(limit)
                .where_()
                .neighbor()
                .query(),
        )
        .query()
}

pub(crate) fn search_last_post_query() -> impl Query + Into<agdb::QueryType> {
    QueryBuilder::search()
        .depth_first()
        .from("posts")
        .limit(1)
        .where_()
        .neighbor()
        .query()
}
