# Guide

In this guide we will explore more realistic use cases and advanced concepts of the `agdb`. It should help you understand how to make the best use of the `graph` data schema and how to build complex queries. The premise of this guide is building a database for a social network called `Myplace`. The users create posts and share them other users for them to react to through commenting and/or liking them.

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

Each post will be connected to `posts` root node and to the user which created it and to users that liked it. Furthermore each post will be connected to comments.

To create a post:

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

Beside connecting the node to two others (doing the edges in a separate steps makes the queries remain fairly simple) we are also adding a property `authored` to the edge coming from the `user`. This is to distinguish it from other possible edges coming from the user - comments and likes - that will come later.

### Likes

- liked

### Comments

- body
