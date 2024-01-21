---
title: "Why not SQL?"
description: "Why not SQL, Agnesoft Graph Database"
navigation:
    title: "Why not SQL?"
---

# Why not SQL?

The following items provide explanations for some of the design choices of `agdb`. All of them are based on research and extensive testing of various approaches and options. For example unlike most graph implementations out there the `agdb` is using pure contiguous vectors instead of linked lists. Curious to lear why? Read on!

-   [Why graph?](#why-graph)
-   [Why not use an existing graph database?](#why-not-use-an-existing-graph-database)
-   [Why object queries?](#why-object-queries)
-   [Why single file?](#why-single-file)
-   [What about sharding, replication and performance at scale?](#what-about-sharding-replication-and-performance-at-scale)

# Why graph?

The database area is dominated by relational database systems (tables) and text queries since the 1970s. However the issues with the relational database systems are numerous and they even gave rise the the regular SW profession - database engineer. This is because contrary to their name they are very awkward at representing actual relations between data which is always demanded by the real world applications. They typically use foreign keys and/or proxy tables to represent them. Additionally the tables naturally enforce fixed immutable data schema upon the data they store. To change the schema one needs to create a new database with the changed schema and copy the data over (this is called database migration). Such operation is very costly and most database systems fair poorly when there are foreign keys involved (requiring them to be disabled for the migration to happen). As it turns out nowadays no database schema is truly immutable. New and changed requirements happen so often that the database schemas usually need updating (migrating) nearly every time there is an update to the systems using it.

There is no solution to this "schema" issue because it is the inherent feature of representing data in tabular form. It can be only mitigated to some degree but your mileage will vary greatly when using these techniques many of which are considered anti-patterns. Things like indexes, indirection (storing data with varied length), storing blob data, data with internal format unknown to the database itself (e.g. JSON) are all ways to prevent the need for database migration at the cost of efficiency. While there are good reasons for representing data in tabular form (lookup speed and space efficiency) the costs of very often far exceed the benefits. Plus as it turns out it is not even that efficient!

The tables are represented as fixed size records (rows) one after another (this is what makes the schema immutable). This representation is the most efficient when we are reading entire rows at the time (all columns) which is very rarely the case. Most often we want only some of the columns which means we are discarding some (or most) of the row when reading it. This is the same problem the CPU itself has when using memory. It reads is using cache lines. If we happen to read only some of the line the rest is wasted and another line needs to be fetched for the next item(s) (this is called a `cache miss`). This is why contiguous collections (like a `vector`) are almost always the most efficient because they minimize the cache misses. Chandler Carruth had numerous talks at CPPCon on this subject demonstrating that by far the biggest performance impact on software are the cache misses (over 50 % and up to 80 % !!!) with everything else being dwarfed in comparison.

Beside trying to optimize the tables the most prominent "solution" are the NoSQL databases. They typically use a different way to store data, often in a "schema-less" to cater to the above use cases - easing database migrations (or eliminating them) and providing more efficient data lookup. They typically choose some combination of key-value representation, document representation or a graph representation to scaffold the data instead of tables. They often trade in ACID properties, use write only - never delete "big tables" and other techniques.

Of NoSQL databases the graph databases stand out in particular because by definition they actually store the relations between the data in the database. How the values are then "attached" to the graph vary but the graph itself serves as an "index" as well as a "map" to be efficiently searched and reason about. The sparse graph (not all nodes are connected to all others) representation is then actually the most flexible and accurate way to store and represent the sparse data (as mentioned nearly or real world data is sparse).

There are two key properties of representing data as a graph that directly relates to the aforementioned issues of schema and data searching. Firstly the graph itself is the schema that can change freely as needed at any time without any restrictions eliminating the schema issue entirely. You do not need to be clairvoyant and agonize over the right database schema. You can do what works now and change your mind later without any issues. Secondly the graph allows accurately representing any actual relations between the data allowing the most efficient native traversal and lookup of data (vaguely resembling traditional indexing) making the lookup constantly efficient regardless of the data set size. Where table performance will deteriorate as it grows the graph will stay constantly efficient if you can traverse only the subset of the nodes via their relations even if the graph itself contained billions of nodes.

That is in a nutshell why the graph database is the best choice for most problem domains and data sets out there and why `agdb` is the graph database.

**Costs**

Everything has the cost and graph databases are no exception. Some operations and some data representations may be costlier in them as opposed to table based databases. For example if you had immutable schema that never updates then table based database might a better fit as the representation in form of tables is more storage efficient. or if you always read the whole table or whole rows then once again the table based databases might be more performant. Typically though these are uncommon edge cases unlikely to be found in the real world applications. The data is almost always sparse and diverse in nature, the schema is never truly stable etc. On the other hand most use cases benefit greatly from graph based representation and thus such a database is well worth it despite some (often more theoretical) costs.

# Why not use an existing graph database?

The following is the list of requirements for an ideal graph database:

-   Free license
-   Faster than table based databases in most common use cases
-   No new language for querying
-   No text based queries
-   Rust and/or C++ driver
-   Resource efficient (storage & memory)

Surprisingly there is no database that would fit the bill. Even the most popular graph databases such as `Neo4J` or `OrientDB` fall short on several of these requirements. They do have their own text based language (e.g. Cypher for Neo4J). They lack the drivers for C++/Rust. They are not particularly efficient (being mostly written in Java). Even the recent addition built in Rust - `SurrealDb` - is using text based SQL queries. Quite incomprehensibly its driver support for Rust itself is not very mature so far and was added only later despite the system being written in Rust. Something which is oddly common in the database world, e.g. `RethinkDb`, itself a document database, written mostly in C++, has no C++ support but does officially support for example Ruby. Atop of these issues they often do not leverage the graph structure very well (except for Neo4J which does great job at this) still leaning heavily towards tables.

# Why object queries?

The most ubiquitous database query language is SQL which is text based language created in the 1970s. Its biggest advantage is that being text based it can be used from any language to communicate with the database. However just like relational (table) bases databases from the same era it has some major flaws:

-   It needs to be parsed and interpreted by the database during runtime leading to common syntax errors that are hard or impossible to statically check.
-   Being a separate programming language from the client coding language increases cognitive load on the programmer.
-   It opens up the database to attacks from SQL-injection where the attacker is trying to force the interpreter to treat the user input (e.g. table or column names) as SQL code itself issuing malicious commands such as stealing or damaging the data.
-   Being "Turing-complete" and complex language on itself means it can lead (and often leads) to incredibly complex and unmaintainable queries.

The last point is particularly troublesome because it partially stems from the `schema` issue discussed in the previous points. One common way to avoid changing the schema is to transform the data via queries. This is not only less efficient than representing the data in the correct form directly but also increases the complexity of queries significantly.

The solutions include heavily sanitizing the user inputs in an attempt to prevent SQL injection attacks, wrapping the constructing of SQL in a builder-pattern to prevent syntax errors and easing the cognitive load by letting programmers create their queries in their main coding language. The complexity is often being reduced by the use of stored SQL procedures (pre-created queries). However all of these options can only mitigate the issues SQL has.

Using native objects representing the queries eliminate all of the SQL issues sacrificing the portability between languages. However that can be relatively easily be made up via already very mature (de)serialization of native objects available in most languages. Using builder pattern to construct these objects further improve their correctness and readability. Native objects carry no additional cognitive load on the programmer and can be easily used just like any other code.

# Why single file?

All operating systems have fairly low limit on number of open file descriptors for a program and for all programs in total making this system resource one of the rarest. Furthermore operating over multiple files does not seem to bring in any substantial benefit for the database while it complicates its implementation significantly. The graph database typically needs to have access to the full graph at all times unlike say key-value stores or document databases. Splitting the data into multiple files would therefore be actually detrimental. Lastly overall storage taken by the multiple files would not actually change as the amount of data would be the same.

Conversely using just a single file (with a second temporary write ahead log file) makes everything simpler and easier. You can for example easily transfer the data to a different machine - it is just one file. The database can also operate on the file directly if memory mapping was turned off to save RAM at the cost of performance. The program would not need to juggle multiple files and consuming valuable system resources.

The one file is the database and the data.

# What about sharding, replication and performance at scale?

Most databases tackle the issue of (poor) performance at scale by scaling up using replication/sharding strategies. While these techniques are definitely useful and they are planned for `agdb` they should be avoided as much as possible. The increase in complexity when using replication and/or sharding is dramatic and it has adverse performance impact meaning it is only worth it if there is no other choice.

The `agdb` is designed so that it performs well regardless of the data set size. Direct access operations are O(1) and there is no limit on concurrency. Write operations are O(1) amortized however they are exclusive - there can be only one write operation running on the database at any given time preventing any other read or write operations at the same time. You will still get O(n) complexity when searching the (sub)graph as reading a 1000 connected nodes will take 1000 O(1) operations = O(n) same as reading 1000 rows in a table. However if the data does not indiscriminately connect everything to everything one can have as large data set as the hardware can fit without performance issues. The key is querying only subset of the graph (subgraph) since your query will have performance based on that subgraph and not all the data stored in the database.

The point here is that scaling has significant cost regardless of technology or clever tricks. Only when the database starts exceeding limits of a single machine they shall be considered because adding data replication/backup will mean huge performance hit. To mitigate it to some extent caching can be used but it can never be as performant as local database. The features "at scale" are definitely coming you should avoid using them as much as possible even if available.

[For real world performance see dedicated documentation.](performance.md)
