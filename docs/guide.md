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
