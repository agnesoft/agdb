# Guide

In this guide we will explore more realistic use cases and advanced concepts of the `agdb`. It should help you understand how to make the best use of the `graph` data schema and how to build complex queries. The premise of this guide is building a database for a social network called `Myplace`. The users create posts and share them other users for them to react to through commenting and/or liking them.

- [Guide](#guide)
  - [The setup](#the-setup)
    - [Users](#users)
    - [Posts](#posts)
    - [Comments](#comments)
    - [Likes](#likes)
  - [Selects \& Searches](#selects--searches)
    - [Login](#login)
    - [User posts, comments \& liked content](#user-posts-comments--liked-content)
  - [Schema Updates](#schema-updates)
  - [Summary](#summary)

## The setup

The first thing is to setup our database:

```Rust
use std::sync::{Arc, RwLock};

let db = Arc::new(RwLock::new(Db::new("myplace.agdb")?));
```

We are setting it up for the multithread use with the `Arc` and the `RwLock`. The initial setup will be done solely in the main thread but later queries would be offloaded to threads to leverage unlimited read parallelism that `agdb` allows.

```Rust
db.write()?.transaction_mut(|t| -> Result<(), QueryError> {
    t.exec_mut(&QueryBuilder::insert().nodes().aliases(vec!["root", "users", "posts"]).query())?;
    t.exec_mut(&QueryBuilder::insert().edges().from("root").to(vec!["users", "posts"]).query())?;
    Ok(())
})?;
```

First we crete some basic nodes. One for `users` and one for `posts`. We also create `root` node and connect the other two to it. It serves a few purposes. The `agdb` does allow disjointed graphs but it is not easy to navigate an unknown database (e.g. when opening the database file in a data editor/explorer). A useful convention is thus to specify a root node. If it is a first node (that would always end up with the `id` == `1`) or has an alias (i.e. `root`) the entrypoint is known. We can then connect other root nodes of our sub-graphs to it. Or insert their aliases as properties to the `root` node.

NOTE: There is no preferred or even hardcoded method to do this which is intentional. You may also choose not to do it at all if you do not have a need for data discovery.

With this simple basic setup it's time to think about how we want to model our data given we have a graph to work with. Right off the bat the nice think about it is tha we do not need to think too hard about it. Going with what feels natural is the right choice because changing it later is actually very easy and will feel natural as well.

### Users

The users of our social network will be nodes connected to the root `users` node. The information we want to store about our users are:

- username
- e-mail
- password

Super basic set of properties but then again we can always expand (or reduce!) it later. A query creating the user could then look like this:

```Rust
use agdb::DbId;

let username = "luckyjoe";
let email = "lucky.joe@internet.net";
let password = "mypassword123";

let user = db.write()?.transaction_mut(|t| -> Result<DbId, QueryError> {
    let user = t.exec_mut(&QueryBuilder::insert().nodes().values(vec![vec![
                    ("username", username).into(),
                    ("email", email).into(),
                    ("password", password).into()
                ]]).query())?.elements[0].id;
    t.exec_mut(&QueryBuilder::insert().edges().from("users").to(user).query())?;
    Ok(user)
})?;
```

Notice we once again used a transaction to create a node and connect it to the graph immediately. The reason why this is done in two steps (queries) is to keep the queries simpler (to reduce cognitive load) and because we want to get back the result of the node insertion - the user id - for which we will use the database element id.

### Posts

The actual content our users create and consume are the posts. The properties we would want to store about the post are:

- title
- body
- author

Each post will be connected to `posts` root node and to the user which created it. To create a post:

```Rust
use agdb::QueryId;

let title = "My awesome car";
let body = format!("https://photos.myplace.net/{}/car.jpeg", user.0);
let timestamp = 123;

let post = db.write()?.transaction_mut(|t| -> Result<DbId, QueryError> {
    let post = t.exec_mut(&QueryBuilder::insert().nodes().values(vec![vec![
                    ("title", title).into(),
                    ("body", body.clone()).into(),
                ]]).query())?.elements[0].id;
    t.exec_mut(&QueryBuilder::insert().edges().from(vec![QueryId::from("posts"), user.into()]).to(post)
                    .values(vec![vec![],vec![("authored", timestamp).into()]]).query())?;
    Ok(post)
})?;
```

Beside connecting the node to two others (doing the edges in a separate step makes the queries remain fairly simple) we are also adding a property `authored` to the edge coming from the `user`. This is to distinguish it from other possible edges coming from the user - comments and likes - that will come later.

### Comments

The comments are created by the users and are either top level comments on a post or replies to other comments. Information we want to store about the comments are:

- body
- author

To create a top level comment:

```Rust
let body = "I have this car as well only in red. It's a great car!";
let timestamp = 456;

let top_comment = db.write()?.transaction_mut(|t| -> Result<DbId, QueryError> {
    let comment = t.exec_mut(&QueryBuilder::insert().nodes().values(vec![
        vec![("body", body).into()]]).query())?.elements[0].id;
    t.exec_mut(&QueryBuilder::insert().edges().from(vec![post, user]).to(comment).values(vec![
        vec![], vec![("commented", timestamp).into()]]).query())?;
    Ok(comment)
})?;
```

Similarly to create a reply to a comment:

```Rust
let body = "They stopped making them just a year later in 2009 and the next generation flopped so they don't make them anymore. It's a shame, it really was a good car.";
let timestamp = 789;

let reply_comment = db.write()?.transaction_mut(|t| -> Result<DbId, QueryError> {
    let comment = t.exec_mut(&QueryBuilder::insert().nodes().values(vec![
        vec![("body", body).into()]]).query())?.elements[0].id;
    t.exec_mut(&QueryBuilder::insert().edges().from(vec![top_comment, user]).to(comment).values(vec![
        vec![], vec![("commented", timestamp).into()]]).query())?;
    Ok(comment)
})?;
```

The edges from the user have now a property `commented` to distinguish them from `authored` edges. Alternatively one could create a proxy node (i.e. "user comments") that would identify all edges coming from it as a particular type eliminating the need to check all the edges. We will see it in action a bit later but as with any optimization technique we should measure its impact on what we are doing.

### Likes

Let's make the `likes` connections from users to posts or comments. To create likes:

```Rust
let timestamp = 246;

db.write()?.exec_mut(
    &QueryBuilder::insert()
        .edges()
        .from(user)
        .to(vec![post, top_comment])
        .values_uniform(vec![("liked", timestamp).into()])
        .query(),
)?;
```

The query is fairly self-explanatory. The `likes` have the `liked` property that distinguishes it from the other edges from a user.

## Selects & Searches

Now that we have the data in our database and means to add more it is time to create the select and search queries.

### Login

First the user login which means searching the database for a particular username and matching its password:

```Rust
use agdb::Comparison::Equal;
use agdb::CountComparison::LessThanOrEqual;

let user = db.read()?.exec(
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
            .query())?
    .elements
    .get(0)
    .ok_or(QueryError::from("Username or password are incorrect"))?
    .id;
```

There is a lot going on here. First we start our search at the `users` using depth first. Depth first is better here because it allows us to examine users in sequence rather than first examining all edges from users and only then examining all the users. We also limit the search to just a single element (`limit(1)`) as we want just that one user. The conditions are then straightforward:

- go at most to distance 2 (distance 1 = edges from users, distance 2 == user nodes)
- check if username matches
- check if password matches

Upon success we attempt to get the first element in the result and its id.

### User posts, comments & liked content

Showing a user their content or content they liked in the past would be a very typical query we would like to have:

```Rust
let user_posts = db.read()?.exec(
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
```

As usual we start the search with `from`, this time from the logged in `user` node. We know the posts are at distance 2 from there (beyond single edge) so the first condition is `distance(Equal(2))`. And we want to skip the edges that are not leading to a post (i.e. those not having the `authored` property or key). Finally we want to continue searching beyond the starting node hence the condition for the `beyond` modifier is compounded condition - either a node or an element with the `authored` key.

We would retrieve the comments and likes in a similar fashion only changing the `keys` condition to `commented` or `liked` respectively.

## Schema Updates

TBD

## Summary

TBD
