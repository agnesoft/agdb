---
title: "Efficient agdb"
description: "Efficient agdb, Agnesoft Graph Database"
---

# Efficient agdb

In this document we will explore more realistic use of the `agdb`. It should help you understand how to make the best use of the `graph` data schema and how to build complex queries.

The premise that we will be working on is building a database for a social network. The users of the network can create posts and share them with other users to comment and like. You can see the complete code under [tests/efficient_agdb.rs](https://github.com/agnesoft/agdb/blob/main/agdb/tests/efficient_agdb.rs).

## The setup

```rs
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
```

We are setting up the database for the multithreaded use with the `Arc` and the `RwLock` in order to leverage unlimited read parallelism. We create nodes for `users` and one for `posts` and create a `root` node and connect the other two to it. The `agdb` does allow disjointed graphs, but it is not easy to navigate an unknown database (e.g. when opening the database file in a data editor/explorer without knowing its content). A useful convention is thus to specify a root node. If it is a first node (that would always end up with the `id` == `1`) or has an alias (i.e. `root`) the entry-point is known. We can then connect other nodes to it or insert their aliases (or `ids`) as properties to the `root` node. There is no preferred or hard-coded method to do this which is intentional. You may also choose not to do it at all if you do not have a need for data discovery.

### Users

The users of our social network will be nodes connected to the `users` node. The information we want to store about our users are:

-   username
-   e-mail
-   password

Lets firs define the `User` struct to hold this information:

```rs
#[derive(UserValue)]
struct User {
    username: String,
    email: String,
    password: String,
}
```

We derive from `agdb::UserValue` so we can use the `User` type directly in our queries. A query creating the user would therefore look like this:

```rs
fn register_user(db: &mut Db, user: &User) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
        if t.exec(
            QueryBuilder::search()
                .from("users")
                .where_()
                .key("username")
                .value(Equal(user.username.into()))
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
            .exec_mut(
                QueryBuilder::insert()
                    .element(user)
                    .query(),
            )?
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
```

First we check if the user exists and return error if the username is taken. We then use a transaction to create a user node and edge from `users` node. The reason why this is done in two steps (queries) is to keep the queries simpler and because we want to get back the result of the node insertion - the user `id`. If we fed the insert nodes query to the insert edges query the `id` would be lost.

### Posts

The users should be able to create posts. The data we want to store about the posts are:

-   title
-   body
-   author

Once again lets define the `Post` type. The specially treated `db_id` field will become useful later on:

```rs
#[derive(UserValue)]
struct Post {
    db_id: Option<DbId>,
    title: String,
    body: String,
}
```

The first two will become properties while the `author` will be represented as an edge. To create a post:

```rs
fn create_post(db: &mut Db, user: DbId, post: &Post) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
        let post = t
            .exec_mut(
                QueryBuilder::insert()
                    .element(post)
                    .query(),
            )?
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
```

Beside connecting the node to two others we are also adding a property `authored` to the edge coming from the `user`. This is to distinguish it from other possible edges coming from the user - comments and likes.

### Comments

The comments are created by the users and are either top level comments on a post or replies to other comments. Information we want to store about the comments are:

-   body
-   author
-   parent (post OR comment)

We define the comment type:

```rs
#[derive(UserValue)]
struct Comment {
    body: String,
}
```

To create a comment:

```rs
fn create_comment(
    db: &mut Db,
    user: DbId,
    parent: DbId,
    comment: &Comment,
) -> Result<DbId, QueryError> {
    db.transaction_mut(|t| -> Result<DbId, QueryError> {
        let comment = t
            .exec_mut(
                QueryBuilder::insert()
                    .element(comment)
                    .query(),
            )?
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
```

The `parent` parameter can be either a post `id` or a comment `id`. The edges from the user have now a property `commented` to distinguish them from `authored` edges.

### Likes

Likes can be best modelled as connections from users to posts and comments:

```rs
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
```

The query is fairly self-explanatory. The edge has the `liked` property that distinguishes it from the other edges from a user (i.e. `authored` and `commented`).

Since users can decide that they no longer like a post or comment we need to have the ability to remove it:

```rs
fn remove_like(db: &mut Db, user: DbId, id: DbId) -> Result<(), QueryError> {
    db.transaction_mut(|t| -> Result<(), QueryError> {
        t.exec_mut(
            QueryBuilder::remove()
                .search()
                .from(user)
                .to(id)
                .where_()
                .keys(vec!["liked".into()])
                .query(),
        )?;
        Ok(())
    })
}
```

This query removes elements returned by the search. The search is the "path search" starting (`from`) the user and looking for the `id` (`to`). It selects only the element with the `liked` property which would be the edge we are looking for. The query is simple because it takes advantage of several facts:

-   if the `id` exists the path to it will contain 3 elements: starting node, an edge and the `id` node
-   elements not selected for the result by the condition are penalized in the path search eliminating the candidate path through the `authored` node
-   `limit(1)` is not useful here because path search applies the limit after it found the best path which would be as described - containing just one suitable element anyway

Still if we were unsure the `id` exists or if we wanted to limit the search area as much as possible we could create a chain of conditions to only restrict the search to a particular distance and prevent the other edges to be followed:

```rs
.where_()
.distance(CountComparison::LessThanOrEqual(2))
.and()
.keys(vec!["liked".into()])
.and()
.beyond()
.where_()
.keys(vec!["liked".into()])
.or()
.node()
```

While this condition is more robust it is also harder to follow, particularly the `beyond()` part where the function of the `.or().node()` might not be immediately obvious. It has to be there because otherwise the `beyond()` condition would prevent the starting node to be followed as well since it does not have the `liked` property, and we only want the search to follow those (and naturally the starting node).

## Selects & Searches

Now that we have the data in our database and means to add (or remove) more it is time to create the select and search queries. Recall that the search queries find the `ids` of the database (graph) elements. To read properties the properties you would need to combine it with a `select` query.

### Login

First the user login which means searching the database for a particular username and matching its password:

```rs
fn login(db: &Db, username: &str, password: &str) -> Result<DbId, QueryError> {
    let result = db
        .exec(
            QueryBuilder::select()
                .values("password".into())
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
```

First we retrieve the password if the user exists:

1. Start the search at the `users` node using depth first algorithm. The depth first is better here because it allows us to examine users in sequence rather than first examining all the edges from the `users` node and only then all the users.
2. Limit the search to just a single element (`limit(1)`) as we want just one user, and we want to stop once it is found.
3. Limit the distance of the search to elements at distance 2 (distance 0 = starting node, distance 1 == edges from users, distance 2 == user nodes).
4. Check `username` property for a match against the passed in username.

Upon success, we attempt to get the first element in the result returning "user not found" if it is not there. Finally, we get the value of the password (we have selected single property so we know it is there) from the result and check if the password matches.

You may be wondering why we do not check the password in the query directly. The reason is that we have no way of stopping the further search if only the `username` matched but not the `password`. The search would then needlessly continue over all users. Therefore, we only retrieve the password and match it in the code. Since it would be salted and hashed anyway it would not be possible to do it with a database query in the first place.

### User content

Showing users their content or content they liked can be done with a following query first retrieving the `ids` of the posts:

```rs
fn user_posts_ids(db: &Db, user: DbId) -> Result<Vec<QueryId>, QueryError> {
    Ok(db
        .exec(
            QueryBuilder::search()
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
```

This time we search from the logged in `user` node identified by its `id` (i.e. returned from the `login()` function). We know the posts are at distance 2 from there (beyond just a single edge) so the first condition is `distance(Equal(2))`. We narrow down the search further with `beyond()` and a condition that limits where we want the search to go. That is the purpose of the `keys(vec!["authored".into()])` so only elements with `authored` key are followed. Unfortunately that would exclude the starting user node (user nodes do not have the `authored` property) so we add `or().node()` condition to continue search from nodes as well. In this case it would apply only to the very first node, but that is exactly what we need.

The same outcome can be reached with number of other conditions as well. For example using `keys(vec!["title".into()])` condition. However, it would also examine all the comments and likes (you may remember similar discussion in the method to [remove likes](#likes)). Another option could be using `not_beyond()` with `where_().keys(vec!["commented"]).or().keys(vec!["liked"])` - explicitly stopping at edges with those properties (`commented` and `liked`). The `keys()` condition is "all or nothing" so it needs to be `or`ed and specified twice in this case.

Similarly to `user_posts` we can fetch the user comments and liked posts with slight modification of the condition:

-   user comments: `.keys(vec!["commented".into()])`
-   liked posts: `keys(vec!["title".into()])` and `.keys(vec!["liked".into()])`

Notice as well that the function returns the `ids` of the elements we were interested in which gives us flexibility in what we want to retrieve about the posts. In order to retrieve say titles of the posts we would need to feed it to a select query:

```rs
fn post_titles(db: &Db, ids: Vec<QueryId>) -> Result<Vec<String>, QueryError> {
    Ok(db
        .exec(
            QueryBuilder::select()
                .values("title".into())
                .ids(ids)
                .query(),
        )?
        .elements
        .into_iter()
        .map(|post| post.values[0].value.to_string())
        .collect())
}
```

Here we take advantage of the fact that we have selected a single property so that every element in the result is guaranteed to have it.

### Posts

Selecting all posts is a fairly straightforward query, but we would rarely need all of them at once. A common need for large collections of data is "paging". That means returning only a chunk of data at a time. Similarly to SQL we can use both `offset` and `limit` to achieve this:

```rs
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
```

By running the function repeatedly and incrementing the `offset` by the `limit` we would iterate over all posts in `limit` steps (usually called "pages"). Notice the `distance` condition which is all we need to limit the search to just posts.

There is something missing though as we would want to also order the posts by the number of likes they have. That would be possible with the current schema but not very easily. We will revisit this a later when we will discuss the schema updates.

### Comments

Now that we have the posts we will want to fetch the comments on them. Our schema says that the only outgoing edges from posts are the comments so getting the comments can be done like this:

```rs
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
```

Using the `depth_first` algorithm will help in organizing the comments in their natural tree structure in the result. The comments are nodes so `.node()` is the first condition. We are starting at the post, but we are not interested in selecting that hence the condition `.distance(CountComparison::GreaterThan(1))`. Since we are selecting the `body` property we can assume it when extracting it to a vector of comments.

There is another flaw here however, do you see it? We currently do not have a way to tell which comment is a top level comment to correctly present the comments to the user other than in a flat list. This is another case for a schema update to satisfy this requirement.

## Schema updates

Possibly the most common problem with any database is that it contains the information we want in some form, but it does not allow us using it in the way we would like. Perhaps we want to join the information together or get it in a different format than in which it is stored. Or the information is not there, and we need to start capturing it. These issues are not unique to `agdb` or to databases in general for that matter. They are ubiquitous in all software as requirements and our understanding of the problem domains change over time. The ability to change is what matters the most. Let's see how `agdb` tackles it.

In our case we have already identified two such issues with our database so far:

-   ordering posts based on likes
-   determining level of comments

We can perhaps already come up with more such as getting the authors of posts or comments, missing timestamp information etc. There are certainly more but for now let's focus on the two highlighted ones:

### Likes

Let's start with the likes. The query to make use of the `liked` edges would not be terribly difficult (counting the `liked` edges incoming to a post or comment) but it certainly does not seem that easy or fast. Especially as we would be doing it over and over. Instead, we could simply introduce a counter property called `likes` and essentially cache the information on the posts (or comments) themselves. That would simplify and speed up things:

```rs
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
                        .keys(vec!["liked".into()])
                        .query(),
                )?
                .result;
            likes.push(vec![("likes", post_likes).into()]);
        }

        t.exec_mut(QueryBuilder::insert().values(likes).ids(posts).query())?;
        Ok(())
    })
}
```

We are doing a mutable transaction to prevent any new posts, likes or other modifications to interfere while we do this. First we get the `ids` of all the posts. Then we count the `liked` edges of each post (exactly what we would be doing if we did not want to change the schema) and finally we insert a new `likes` property with that count back to the posts. Furthermore, we should update our definition of `Post`:

```rs
#[derive(UserValue)]
struct PostLiked {
    db_id: Option<DbId>,
    title: String,
    body: String,
    likes: i64,
}
```

This allows us to select and order posts based on likes:

```rs
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
```

However this change is not "free" in that caching any information means it now exists in two places and those places must be synchronized (the famous cache invalidation problem). Fortunately this instance is not as hard. We simply make sure that whenever we add or remove `likes` we also update the counter on the post or comment. Since when we do those operations we also have the post/comment `id` doing that would be trivial.

### Comments

Another issue we found was that comments do not track their level, and we cannot present them hierarchically. To make things simpler let's add only a simple distinction between top level comments and replies disregarding any further nesting. To do that we would simply mark the top level comments with a new property:

```rs
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
```

Although this task might have seemed daunting it could be done with a simple query that once more takes advantage of the graph schema. We know that the only outgoing edges from the posts are the comments and that they are hierarchical (replies are attached to the comments they reply to). Therefore, when searching from the `posts` node at distance `4` we will find only the top level comments. We then uniformly apply a property `level=1` to them. Such a property could then be used by the client code to determine how the data is displayed to the users.

## Summary

In this guide we have gone through a realistic example of an inception of a database setting it up from scratch, designing search queries and leveraging the graph schema. We have used the ability to limit the search area based on our data and the graph schema multiple times. Instead of searching possibly millions of records and filtering them out to get what we want we could search & select just the relevant fraction of the data set. That is the main advantage of the graph databases. If a user authored just 3 posts the query would do exactly the same work if there were 30 posts total in the database as if there were 3 billion.

We have also discovered issues with the schema and were able to seamlessly fix them. It demonstrated yet another important aspect of graph databases which is fearless schema updates. Modelling data on a graph feels natural and changing it to fit new or changing requirements is just as natural.

Lastly we have seen that the queries can be simple, readable, statically checked and completely native while still providing complex functionality such as filtering through conditions, paging, ordering etc. Moreover, while the features of object queries won't make them always logically correct they eliminate entire categories of issues like syntax errors, type errors, security issues like SQL injection, and even certain logic errors etc.

For the comprehensive overview of all queries see the [query reference](/docs/references/queries). For the code used in this document see [tests/efficient_agdb.rs](https://github.com/agnesoft/agdb/blob/main/agdb/tests/efficient_agdb.rs).
