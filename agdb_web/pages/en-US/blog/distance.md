---
title: Distance"
description: "Blog, Agnesoft Graph Database"
---

import { Callout } from "nextra/components";

# Distance

The `distance` is a unique property of graphs and perhaps their most important feature. Let's look at the series of cases that will demonstrate what distance actually is and how it is useful when working and thinking about data on the graph. First let's consider the following simplest graph:

![distance - simple](/images/distance_simple.png)

We have two nodes - "projects" and "project1" - connected with an edge. The distance is only relevant when we are searching the graph and is always measured from the point of origin of our search. In this case we search from "projects" and therefore the "projects" node is at distance 0. The adjacent edge is at distance 1 and finally the "project1" is at distance 2.

The distance is also relative to the direction of the search. Consider a reverse search of the above graph:

![distance - simple - reversed](/images/distance_simple_reversed.png)

Here the origin node is "project1" and therefore it is at distance 0. The edge is once again at distance 1 and the "projects" node is at distance 2.

## Distance of 2

When searching graphs perhaps the most useful common distance we would be interested in is distance of 2. Why? Because at distance 2 there will always be all the neighboring nodes. Continuing with our example if we follow a typical pattern of having a `root` node (such as "projects") being connected to individual member nodes (individual projects) we could mimic a "table" in a relational database:

![distance of 2](/images/distance_2.png)

In `agdb` the query to find all projects would look like this (using `Rust` as the query language):

```rust
QueryBuilder::search().from("projects").where_().distance(CountComparison::Equal(2)).query();
```

Searching from the node "projects" and only returning elements at distance 2. The distance here serves two purposes. One is that it tells the algorithm what to return (element at distance 2) but even more importantly it tells it when to stop. Since we are asking only for elements at the distance 2 it knows that it does not need to consider anything beyond that distance and can safely ignore it!

## Limit the search

In the previous section we have hinted to the power of distance so let's explore it properly. Expanding our graph with outputs of our projects: 

![distance - outputs](/images/distance_outputs.png)

The previous query searching at distance 2 remains unaffected. It would still stop searching at distance 2 regardless if there are further elements connected beyond that distance. It therefore does not matter how large and complicated the graph is because with `distance` we can ignore it all and focus the search to only the relevant `subgraph` that we are interested in. Even if the graph had a billion nodes, the above query confined to this subgraph would perform the same.

Another way the `distance` limits the required work when searching the graph is that even if the element is reached via the distance constrained - node "projects" at distance 0 and all the connected edges - because it cannot be included in the final result the algorithm will not be examining its properties if additional constraints are defined. For instance searching for a particular project:

```rust
QueryBuilder::search().from("projects").where_().distance(CountComparison::Equal(2)).and().key("name").value(Comparison::Equal("project1".into())).query();
```

The search algorithm will only look at the "name" properties of the elements at distance 2 rather than all properties. It is generally a good idea to lead the conditions with the distance constraints for this very reason.

The more efficient version would be to switch to depth-first-search and limit the search to a single returned element as to further restrict the work required:

```rust
QueryBuilder::search().depth_first().from("projects").limit(1).where_().distance(CountComparison::Equal(2)).and().key("name").value(Comparison::Equal("project1".into())).query();
```

## No more joins

One of the most ubiquitous actions done in relational databases is joining the data from multiple tables. If we were to model the graph in a relational database we would likely end up with two tables - one for "projects" and one for "outcomes". We would additionally need a link (relation) between them that would most likely be accomplished with a foreign key column in "projects" referencing rows in the "outputs" table. In the query we would then use a join to extract data from both tables. On a graph this is much simpler because the relations are directly represented and joins are not necessary. Once more we can utilize the distance to get the outputs:

```rust
QueryBuilder::search().from("projects").where_().distance(CountComparison::Equal(4)).query();
```

Since we know the structure of our graph we also know that the outputs are at distance 4 and can leverage that information to easily extract all outputs. Furthermore, if we wanted only outputs of a particular project then we would additionally use the `beyond` condition like so:

```rust
QueryBuilder::search()
    .from("projects")
    .where_()
        .distance(CountComparison::Equal(4))
        .and()
            .beyond()
            .where_()
                .distance(CountComparison::NotEqual(2))
                .or()
                .key("name")
                .value(Comparison::Equal("B".into()))
    .query();
```

Let's break it down:

- search starts at the node "projects" (distance 0)
- we want only elements at distance 4 (the outputs): `.distance(CountComparison::Equal(4))`
- additionally we want to limit the search to continue only `beyond` elements that satisfy the next condition: `.and().beyond().where_()` (NOTE: the nested `.where_()` is like opening a bracket)
- the beyond conditions are: continue only if the distance is NOT 2 (`.distance(CountComparison::NotEqual(2))`) or if the element has property "name" with the value "B"

The `beyond` condition might seem alien but when you look at the picture of the graph it should become clear. We are simply telling the algorithm to not consider anything at distance 2 (and therefore beyond) unless the `or` condition is satisfied (the element has the name we are looking for). Because only one node in our graph satisfies such condition ("project B") the algorithm will continue beyond it, reach the outputs (at distance 4) and return only those. Once more potentially greatly limiting the scope of work needed.

The detailed steps taken by the algorithm and how it evaluates each element:

1. Distance 0: distance is less than 4 and the first condition evaluates `true`, we are also not at distance 2 so the beyond condition also evaluates `true` and not even bother with the `key` condition.
2. Distance 1: same evaluation as distance 0
3. Distance 2: distance is still less than 4 and the first condition evaluates `true`, now we are at distance 2 so the first condition in the beyond conditions evaluates `false` and the algorithm will examine the `key` property of each element at this level evaluating only one as `true`
4. Distance 3: same as 0 and 1, but we reach this distance only via project B as no other node at distance 2 passed the conditions
5. Distance 4: distance is 4 and the first condition evaluates `true` and the algorithm is now stopped as nothing further away can satisfy this condition and evaluation of the beyond conditions is not relevant

For general reference see [conditions evaluation in queries documentation](/docs/references/queries#truth-tables).

<Callout>
    The algorithms will never visit same element twice and are immune to cycles and dead locks.
</Callout>

## Conclusion

As we have seen the `distance` is the powerful property of graphs and is one of the key advantages of modelling data on graphs as opposed to tables. They help us limit the scope of our search and can easily confine the search to small subgraph(s) even if the entire graph consists of billions of elements.
