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
    - [User content](#user-content)
    - [Posts](#posts-1)
    - [Comments](#comments-1)
  - [Schema updates](#schema-updates)
    - [Likes](#likes-1)
    - [Comments](#comments-2)
  - [Summary](#summary)
- [Efficient agdb](#efficient-agdb)
  - [The setup](#the-setup-1)
    - [Users](#users-1)
    - [Posts](#posts-2)
    - [Comments](#comments-3)
    - [Likes](#likes-2)
  - [Selects \& Searches](#selects--searches-1)
    - [Login](#login-1)
    - [User content](#user-content-1)
    - [Posts](#posts-3)
    - [Comments](#comments-4)
  - [Schema updates](#schema-updates-1)
    - [Likes](#likes-3)
    - [Comments](#comments-5)
  - [Summary](#summary-1)

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
                    .values(vec![vec![], vec![("authored", timestamp).into()]]).query())?;
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

Now that we have the data in our database and means to add more it is time to create the select and search queries. Remember that the search queries find the ids of the database (graph) elements. To retrieve the properties you would need to combine it with a `select` query.

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

### User content

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

This time we search from the logged in `user` node. We know the posts are at distance 2 from there (beyond just a single edge) so the first condition is `distance(Equal(2))`. We narrow down the search with `beyond()` and giving it a condition that limits where we want the search to go. That is the `keys(vec!["authored".into()])` so only elements with `authored` key are followed. Unfortunately that would exclude the starting user node (user nodes do not have `authored` property) so we `or()` it with the `node()` condition to follow any nodes as well.

Note that the same outcome can be reached with number of other conditions as well. For example using `keys(vec!["title".into()])` condition. However it would also needlessly examine all the comments and likes as well. Another option could be using `not_beyond()` with `where_().keys(vec!["commented"]).or().keys(vec!["liked"])` - explicitly stopping at edges with those properties (`commented` and `liked`). The `keys()` condition is all or nothing so in that case it needs to be and `or`ed and therefore specified twice.

Selecting user comments would be done in the same way simply swapping the `authored` with `commented`. Liked posts & comments would then be the same query with `liked` instead of `authored`.

As an exercise, how would the query be modified to select only liked posts?

Hint: `keys(vec!["title".into()])`

### Posts

Selecting all posts is a fairly straightforward query but we would rarely need all of them at once. A common need for large collections of data is "paging". That means returning only a chunk of data at a time. Similarly to SQL we can use both `offset` and `limit` to achieve this result:

```Rust
let posts = db.read()?.exec(
    &QueryBuilder::select()
        .ids(
            QueryBuilder::search()
                .from("posts")
                .offset(0)
                .limit(10)
                .where_()
                .distance(CountComparison::Equal(2))
                .query(),
        )
        .query(),
)?;
```

By running the query multiple repeatedly and incrementing the `offset()` we would iterate over all posts 10 at a time. Notice the `distance` condition to limit the search to only area of the graph we want - the posts.

### Comments

Now that we have the posts we will want to fetch the comments. Our graph schema says that the only outgoing edges from posts are the comments so getting the comments could be done with a simple query:

```Rust
let comments = db.read()?.exec(
    &QueryBuilder::select()
        .ids(
            QueryBuilder::search()
                .depth_first()
                .from(posts.elements[0].id)
                .query(),
        )
        .query(),
)?;
```

Using the `depth_first` algorithm will help in organizing the comments in their natural tree structure in the result. There is a flaw here though, do you see it? It will require a schema update to rectify which we will examine next.

## Schema updates

Unfortunately we currently do not have a way to tell which comment is a top level comment to correctly present this data to the user (that is the flaw in our schema we just mentioned). Additionally we would certainly like to select posts based on number of likes or comments. The information is there in the database but it is not easily used for what we want. With some queries we could certainly obtain what we want - for example counting the edges with `liked` property incoming to a post. Or querying comments one at a time and remembering the current nesting level.

These issues are not unique to `agdb`, they are ubiquitous in every database and virtually all software in general. They may be a result of a mistake when designing the schema or they may simply be a new requirement that was not thought of before. Regardless a change to the schema is in order. Luckily in `agdb` this is very simple, in fact it is no different to regular database operation.

### Likes

Lets start with the likes. The query to make use of the `liked` edges would not be terribly difficult but it would certainly be rather expensive for what we want to do. Instead we could simply introduce a property called `likes` and track them that way:

```Rust
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
```

We are doing this as a mutable transaction to prevent any new posts, likes or other modifications to interfere. First we get the ids of all the posts. Then we count the `liked` edges of each post and finally we insert a new `likes` property with that count back to the posts. This will allow us to select and order posts based on likes trivially:

```Rust
use agdb::DbKeyOrder;

let posts = db.read()?.exec(
    &QueryBuilder::select()
        .ids(
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
```

However this change is not "free". Now that we track likes in two different ways we also need to take care of the counter property whenever there are new likes or existing ones are removed (if it is allowed). Such a price might however be acceptable as operations based on the number of likes will far exceed their changes. Nevertheless it is always important to consider the downsides of database schema changes and ideally measure their impact.

### Comments

Another issue we found was that comments do not track their level and we cannot present them hierarchically for example. A simple fix could be to mark the top level comments with a property and use that to display the comments in two level hierarchy only (top level comments + all replies on the same level regardless of nesting):

```Rust
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
```

Although the task might have seemed daunting it was in the end quite simple. We know that the only outgoing edges from the posts are the comments and that they are hierarchical. Therefore when searching from the `posts` node the top level comments will be at distance `4`. We then uniformly apply a property `level=1`. This property is going to be useful more for client code that makes the decisions about the data presentation rather than for the queries.

## Summary

In this guide we have gone through a realistic example of an inception of a database for a social network - setting it up from scratch, designing search queries and leveraging the graph. The ability to limit the search area based on the data itself. Not searching millions of records and filtering them out to get the posts written by a certain user but directly searching only the posts of that user and nothing else. That is the main advantage of a graph database and the main reason why it can scale without limits. If a user authored just 3 posts the query would do exactly the same work if there were 30 posts in the database or 3 billion.

We have also discovered issues with the schema that we were able to easily and seamlessly fix. It demonstrated that working with a database can be easy and fearless. Modelling data on a graph will feel natural because it actually is. You will quickly develop intuition and learn that when something feels hard it indicates there is a gap or a problem with the current schema. And schema updates are normal in `agdb` although even convoluted conditions or running multiple queries to obtain some results are not inherently bad. If they are one off or low frequency queries the schema update might not be the best choice.

Lastly we have seen that the queries can be simple, readable, statically checked and completely native. While it will not make them always 100 % correct it eliminates many categories of issues like syntax errors, type errors, security issues, certain logic errors etc. For the comprehensive overview of all queries see the [query reference](queries.md).

# Efficient agdb

In this document we will explore more realistic use of the `agdb`. It should help you understand how to make the best use of the `graph` data schema and how to build complex queries.

The premise that we will be working on is building a database for a social network. The users of the network can create posts and share them with other users to comment and like. You can see the complete code under [tests/efficient_agdb.rs](../tests/efficient_agdb.rs).

- [Guide](#guide)
  - [The setup](#the-setup)
    - [Users](#users)
    - [Posts](#posts)
    - [Comments](#comments)
    - [Likes](#likes)
  - [Selects \& Searches](#selects--searches)
    - [Login](#login)
    - [User content](#user-content)
    - [Posts](#posts-1)
    - [Comments](#comments-1)
  - [Schema updates](#schema-updates)
    - [Likes](#likes-1)
    - [Comments](#comments-2)
  - [Summary](#summary)
- [Efficient agdb](#efficient-agdb)
  - [The setup](#the-setup-1)
    - [Users](#users-1)
    - [Posts](#posts-2)
    - [Comments](#comments-3)
    - [Likes](#likes-2)
  - [Selects \& Searches](#selects--searches-1)
    - [Login](#login-1)
    - [User content](#user-content-1)
    - [Posts](#posts-3)
    - [Comments](#comments-4)
  - [Schema updates](#schema-updates-1)
    - [Likes](#likes-3)
    - [Comments](#comments-5)
  - [Summary](#summary-1)

## The setup

```Rust
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
```

We are setting up the database for the multithread use with the `Arc` and the `RwLock` in order to leverage unlimited read parallelism. We create nodes for `users` and one for `posts`. We also create a `root` node and connect the other two to it. The `agdb` does allow disjointed graphs but it is not easy to navigate an unknown database (e.g. when opening the database file in a data editor/explorer without knowing its content). A useful convention is thus to specify a root node. If it is a first node (that would always end up with the `id` == `1`) or has an alias (i.e. `root`) the entrypoint is known. We can then connect other nodes to it or insert their aliases (or ids) as properties to the `root` node. There is no preferred or hardcoded method to do this which is intentional. You may also choose not to do it at all if you do not have a need for data discovery.

### Users

The users of our social network will be nodes connected to the `users` node. The information we want to store about our users are:

- username
- e-mail
- password

A query creating the user would therefore look like this:

```Rust
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
```

First we check if the user exists and return error if the username is taken. We then use a transaction to create a user node and edge from `users` node. The reason why this is done in two steps (queries) is to keep the queries simpler and because we want to get back the result of the node insertion - the user id. If we fed the insert nodes query to the insert edges query the id would be lost.

### Posts

The users should be able to create posts. The data we want to store about the posts are:

- title
- body
- author

The first two will become properties while the `author` will be represented as an edge. To create a post:

```Rust
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
```

Beside connecting the node to two others we are also adding a property `authored` to the edge coming from the `user`. This is to distinguish it from other possible edges coming from the user - comments and likes.

### Comments

The comments are created by the users and are either top level comments on a post or replies to other comments. Information we want to store about the comments are:

- body
- author
- post - OR - comment

To create a comment:

```Rust
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
```

The `parent` parameter can be either a post id or a comment id. The edges from the user have now a property `commented` to distinguish them from `authored` edges.

### Likes

Likes can be best modelled as connections from users to posts and comments:

```Rust
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
```

The query is fairly self-explanatory. The edge has the `liked` property that distinguishes it from the other edges from a user (i.e. `authored` and `commented`).

Since users can decide that they no longer like a post or comment we need to have the ability to remove it:

```Rust
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
```

This query removes elements returned by the search. The search is the "path search" starting (`from`) the user and looking for the id (`to`). It selects only the element with the `liked` property which would be the edge we are looking for. The query is simple because it takes advantage of several facts:

- if the `id` exists the path to it will contain 3 elements: starting node, an edge and the `id` node
- elements not selected for the result by the condition are penalized in the path search eliminating the candidate path through the `authored` node
- `limit(1)` is not useful here because path search applies the limit after it found the best path which would be as described - containing just one suitable element anyway

Still if we were unsure the `id` exists or if we wanted to limit the search area as much as possible we could create a chain of conditions to only restrict the search to a particular distance and prevent the other edges to be followed:

```Rust
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

While this condition is more robust it is also harder to follow, particularly the `beyond()` part where the function of the `.or().node()` might not be immediately obvious. It has to be there because otherwise the `beyond()` condition would prevent the starting node to be followed as well since it does not have the `liked` property and we only want the search to follow those (and naturally the staring node).

## Selects & Searches

Now that we have the data in our database and means to add (or remove) more it is time to create the select and search queries. Recall that the search queries find the ids of the database (graph) elements. To read properties the properties you would need to combine it with a `select` query.

### Login

First the user login which means searching the database for a particular username and matching its password:

```Rust
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
```

First we retrieve the password if the user exist:

1. Start the search at the `users` node using depth first algorithm. The depth first is better here because it allows us to examine users in sequence rather than first examining all edges from the `users` node and only then all the users.
2. Limit the search to just a single element (`limit(1)`) as we want just one user and we want to stop once it is found.
3. Limit the distance of the search to elements at distance 2 (distance 1 == edges from users, distance 2 == user nodes).
4. Check `username` property for a match against the passed in username.

Upon success we attempt to get the first element in the result returning "user not found" if it is not there. Finally we get the value of the password (we have selected single property so we know it is there) from the result and check if the password matches.

You may be wondering why we do not check the password in the query directly. The reason is that we have no way of stopping the further search if only the `username` matched but not the `password`. The search would then needlessly continue over all users. Therefore we only retrieve the password and match it in the code.

### User content

Showing users their content or content they liked can be done with a following query:

```Rust
fn user_posts(db: &Db, user: DbId) -> Result<Vec<QueryId>, QueryError> {
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
```

This time we search from the logged in `user` node identified by its id (i.e. returned from the `login()` function). We know the posts are at distance 2 from there (beyond just a single edge) so the first condition is `distance(Equal(2))`. We narrow down the search further with `beyond()` and a condition that limits where we want the search to go. That is the purpose of the `keys(vec!["authored".into()])` so only elements with `authored` key are followed. Unfortunately that would exclude the starting user node (user nodes do not have the `authored` property) so we add `or().node()` condition to continue search from nodes as well (in this case it would apply only to the very first node but that is exactly what we need).

The same outcome can be reached with number of other conditions as well. For example using `keys(vec!["title".into()])` condition. However it would also examine all the comments and likes (you may remember similar discussion in the method to [remove likes](#likes)). Another option could be using `not_beyond()` with `where_().keys(vec!["commented"]).or().keys(vec!["liked"])` - explicitly stopping at edges with those properties (`commented` and `liked`). The `keys()` condition is "all or nothing" so it needs to be `or`ed and specified twice in this case.

Similarly to `user_posts` we can fetch the user comments and liked posts with slight modification of the condition:

- user comments: `.keys(vec!["commented".into()])`
- liked posts: `keys(vec!["title".into()])` and `.keys(vec!["liked".into()])`

Notice as well that the function returns the `ids` of the elements we were interested in. In order to retrieve say titles of the posts we would need to feed it to a select query:

```Rust
fn post_titles(db: &Db, ids: Vec<QueryId>) -> Result<Vec<String>, QueryError> {
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
```

Once more we take advantage of the fact that we have selected a single property so every element in the result is guaranteed to have it.

### Posts

Selecting all posts is a fairly straightforward query but we would rarely need all of them at once. A common need for large collections of data is "paging". That means returning only a chunk of data at a time. Similarly to SQL we can use both `offset` and `limit` to achieve this:

```Rust
fn posts(db: &Db, offset: u64, limit: u64) -> Result<Vec<QueryId>, QueryError> {
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
```

By running the function repeatedly and incrementing the `offset` by the `limit` we would iterate over all posts in `limit` steps (called pages). Notice the `distance` condition which is all we need to limit the search to just posts.

There is something missing though as we would want to also order the posts by the number of likes they have. That would be possible with the current schema but not very easily. We will revisit this a bit later when we will discuss the schema updates.

### Comments

Now that we have the posts we will want to fetch the comments on them. Our schema says that the only outgoing edges from posts are the comments so getting the comments can be done like this:

```Rust
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
```

Using the `depth_first` algorithm will help in organizing the comments in their natural tree structure in the result. The comments are nodes so `.node()` is the first condition. We are starting at the post but we are not interested in selecting that hence the condition `.distance(CountComparison::GreaterThan(1))`. Since we are selecting the `body` property we can assume it when extracting it to a vector of comments.

There is a another flaw here however, do you see it? We currently do not have a way to tell which comment is a top level comment to correctly present the comments to the user other then in a flat list. This is another use case for a schema update to satisfy this requirement.

## Schema updates

Possibly the most common problem with any database is that it contains the information we want in some form but it does not allow us using it in the way we would like. Perhaps we want to join the information together or get it in a different format than in which it is stored. Or the information truly is not there and we need to start capturing it. These issues are not unique to `agdb` or to databases in general for that matter. They are ubiquitous in all software as requirements and our understanding of the problem domains change over time. The ability to change is what matters the most. Let's see how `agdb` tackles this problem.

In our case we have already identified two such issues with our database so far:

- ordering posts based on likes
- determining level of comments

We can perhaps already come up with more such as getting the authors of posts or comments, missing timestamp information etc. There are certainly more but for now let's focus on the two highlighted:

### Likes

Lets start with the likes. The query to make use of the `liked` edges would not be terribly difficult (counting the `liked` edges from a post or comment) but it certainly does not seem that easy or even fast. Especially as we would be doing it over and over. Instead we could simply introduce a counter property called `likes` and essentially cache the information on the post itself. That would simplify and speed up things a lot:

```Rust
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
```

We are doing a mutable transaction to prevent any new posts, likes or other modifications to interfere while we do this. First we get the ids of all the posts. Then we count the `liked` edges of each post (exactly what we would be doing if we did not want to change the schema) and finally we insert a new `likes` property with that count back to the posts. This allows us to select and order posts based on likes:

```Rust
fn liked_posts(db: &Db, offset: u64, limit: u64) -> Result<Vec<QueryId>, QueryError> {
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

```

However this change is not "free" in that caching any information means it now exists in two places and those places must be synchronized (the famous cache invalidation problem). Fortunately this instance is not as hard. We simply make sure that whenever we add or remove `likes` we also update the counter on the post or comment. Since when we do those operations we also have the post/comment id doing that would be trivial.

### Comments

Another issue we found was that comments do not track their level and we cannot present them hierarchically. To make things simpler let's add only a simple distinction between top level comments and replies disregarding any further nesting. To do that we would simply mark the top level comments with a new property:

```Rust
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
```

Although this task might have seemed daunting it could be done with a simple query that once more takes advantage of the graph schema. We know that the only outgoing edges from the posts are the comments and that they are hierarchical (replies are attached to the comments they reply to). Therefore when searching from the `posts` node at distance `4` we will find only the top comments. We then uniformly apply a property `level=1` to them. Such a property could then be used by the client code to determine how the data is displayed to the users.

## Summary

In this document we have gone through a realistic example of an inception of a database for a social network - setting it up from scratch, designing search queries and leveraging the graph. We have used the ability to limit the search area based on our data and the graph schema multiple times. Instead of searching possibly millions of records and filtering them out to get what we want we could search & select just the relevant fraction of the data set. That is the main advantage of the graph databases. If a user authored just 3 posts the query would do exactly the same work if there were 30 posts in the database as if there were 3 billion.

We have also discovered issues with the schema and were able seamlessly fix them. It demonstrated yet another important aspect of graph databases which is fearless schema updates. Modelling data on a graph feels natural and changing it to fit new or changing requirements is just as natural.

Lastly we have seen that the queries can be simple, readable, statically checked and completely native while still providing complex functionality such as filtering through conditions, paging, ordering etc. Moreover while the features of object queries won't make them 100 % correct they eliminate entire categories of issues like syntax errors, type errors, security issues, certain logic errors etc.

For the comprehensive overview of all queries see the [query reference](queries.md). For the code used in this document see [tests/efficient_agdb.rs](../tests/efficient_agdb.rs).
