---
title: "Performance"
description: "Performance, Agnesoft Graph Database"
---

# Performance

Database performance is one of the key metrics when judging the suitability of the solution for a given use case. Individual metrics such as how many inserts or selects can a database handle in a tight loop are not very interesting or indicative of real performance. In this document we will therefore examine the performance of `agdb` in more realistic use cases via `agdb_benchmarks` that simulate real world usage.

The `agdb` is designed with the following principles:

-   ACID database
-   O(1) complexity for direct access
-   O(n) complexity for search
-   Unlimited read concurrency
-   Exclusive writes

The database is ACID compliant, operations must be transactional = `atomic` (A) meaning they are "all or nothing" operations, `consistent` (C) so that the queries will only produce valid state of the data, `isolated` (I) meaning the transactions do not affect each other when in flight and `durable` (D) meaning the database is resistant to system failure and will preserve integrity of the data. Direct access read/write operations have constant complexity of O(1) while search operations are O(n) but the `n` can be limited to a subgraph greatly reducing the time the operation takes.

Let's see if the `agdb` lives up to these principles.

## The benchmark

The `agdb_benchmarks` project is building upon the [Efficient agdb](/docs/references/efficient-agdb) simulating the traffic in a "social network" database. It simulates concurrent read & write operations on the same database:

-   Posters: Writes social media posts
-   Commenters: Writes comments to the existing posts
-   Post readers: Reads existing posts
-   Comment readers: Read existing comments

It is highly configurable through the `agdb_benchmarks.yaml` file (produced on first run) with the following settings:

-   How many of each category of users (post writers, comment writers, post readers, comment readers)
-   How many operations should each user perform
-   How large each operation should be [readers only] (e.g. how many posts to read)
-   Contents of each operation [writes only] (e.g. post title, post body)
-   Delay between each operation

For writers the configured content is additionally augmented by the user `id` to produce unique content. The delays are further shifted by the user `id` to prevent unrealistic resource contention by everyone in a single millisecond. The read operations are repeated if no result is yielded effectively "waiting" for the readers to input data first.

The benchmark uses tokio tasks spawning everything together. It measures each database operation (transaction as some operations are multiple queries) for minimum, average, maximum and total elapsed time. Additionally, it shows total database size after all operations finished and furthermore after running the optimization algorithm compacting (defragmenting) the data.

### Default settings

-   Insert user nodes (for post & comment writers)
-   10 post writers (100 posts each, 100ms delay, non-small title (>15 bytes) & body (>15 bytes))
-   10 comment writers (100 comments each, 100ms delay, non-small body (>15 bytes))
-   100 post readers (100 reads each, 10 posts per read, 100ms delay)
-   100 comment readers (100 reads each, 10 comments per read, 100ms delay).

### Measured operations

-   Insert user nodes: a node aliased `"users"` with individual users connected to it with a blank edge. Each user has properties `"name"` and `"email"` (values small values `<15 bytes`).
-   Write posts: a post node connected with a blank edge to the single node aliased `"posts"` and with an edge (property `"authored": 1`) to the respective user node. The properties are `"title"` and `"body"` from config (values are large `>15 bytes`).
-   Write comments: a comment node connected with a blank edge to the latest post (found via search from `"posts"` node) and with an edge (property `"commented": 1`) to the respective user node. The properties are only `"body"` from config (value is large `>15 bytes`).
-   Read posts: reads configured amount (e.g. 10 by default) of recent posts on each iteration (found via search from `"posts"` node).
-   Read comments: reads configured amount (e.g. 10 by default) of recent comments on the latest post (found via search from `"posts"` node).
-   Database size: after all operations finished & after optimization algorithm is run.

### Run command

```
cargo run --release -p agdb_benchmarks
```

## Results

The following benchmarks were run on:

-   CPU: Intel Core i7-7700 4 cores (8 logical cores) @ 3,6 GHz
-   RAM: Crucial Ballistix Sport LT 16GB (2x8GB) DDR4 @ 2400 MHz
-   DISK: HyperX Savage - 240GB (KINGSTON SHSS37A240G, 4 cores, 8 channels Phison S10, 560 MB/s read, 530 MB/s write, SATA III (6 Gb/s))
-   OS: Windows 10 22H2 (19045.3448), Debian: Version 12 (bookworm) [running in Hyper-V/WSL2]

When running on a different machine your results will vary, but the relative comparisons should still hold.

### Memory mapped (default)

The benchmark run with [default settings](#default-settings) using memory mapped file persistent storage (database size is limited to available RAM):

**Windows**

| Description    | Threads  | Iters  | Per iter | Count  | Min   | Avg    | Max    | Total |
| -------------- | -------- | ------ | -------- | ------ | ----- | ------ | ------ | ----- |
| Creating users | 1        | 1      | 20       | 20     | -     | 1 ms   | -      | 30 ms |
| Write posts    | 10       | 100    | 1        | 1 000  | 1 ms  | 25 ms  | 3 s    | 8 s   |
| Write comments | 10       | 100    | 1        | 1 000  | 1 ms  | 29 ms  | 3 s    | 8 s   |
| Read posts     | 100      | 100    | 10       | 10 000 | 14 μs | 387 μs | 282 ms | 9 s   |
| Read comments  | 100      | 100    | 10       | 10 000 | 9 μs  | 295 μs | 21 ms  | 9 s   |
| Database size  | 1 627 kB | 785 kB |          |        |       |        |        |       |

**Debian (Hyper-V/WSL2)**

| Description    | Threads  | Iters  | Per iter | Count  | Min    | Avg    | Max    | Total |
| -------------- | -------- | ------ | -------- | ------ | ------ | ------ | ------ | ----- |
| Creating users | 1        | 1      | 20       | 20     | -      | 316 μs | -      | 6 ms  |
| Write posts    | 10       | 100    | 1        | 1 000  | 390 μs | 1 ms   | 141 ms | 2 s   |
| Write comments | 10       | 100    | 1        | 1 000  | 397 μs | 1 ms   | 286 ms | 1 s   |
| Read posts     | 100      | 100    | 10       | 10 000 | 12 μs  | 716 μs | 287 ms | 5 s   |
| Read comments  | 100      | 100    | 10       | 10 000 | 6 μs   | 445 μs | 286 ms | 5 s   |
| Database size  | 1 627 kB | 785 kB |          |        |        |        |        |       |

---

The data shows that the average write operation without contention is very fast (Creating users). Concurrent writes that also contest the database with read operations increase the latency by an order of magnitude. The read operations that can be as fast as <10μs can slow down with contention up to two orders of magnitude particularly due to frequent reads.

### File only

The benchmark run with [default settings](#default-settings) using file persistent storage only (no memory use but unlimited database size):

**Windows**

| Description    | Threads  | Iters  | Per iter | Count  | Min    | Avg    | Max    | Total |
| -------------- | -------- | ------ | -------- | ------ | ------ | ------ | ------ | ----- |
| Creating users | 1        | 1      | 20       | 20     | -      | 1 ms   | -      | 38 ms |
| Write posts    | 10       | 100    | 1        | 1 000  | 1 ms   | 650 ms | 96 s   | 306 s |
| Write comments | 10       | 100    | 1        | 1 000  | 1 ms   | 1 s    | 149 s  | 306 s |
| Read posts     | 100      | 100    | 10       | 10 000 | 604 μs | 23 ms  | 758 ms | 305 s |
| Read comments  | 100      | 100    | 10       | 10 000 | 390 μs | 28 ms  | 775 ms | 304 s |
| Database size  | 1 627 kB | 785 kB |          |        |        |        |        |       |

**Debian (Hyper-V/WSL2)**

| Description    | Threads  | Iters  | Per iter | Count  | Min    | Avg    | Max    | Total |
| -------------- | -------- | ------ | -------- | ------ | ------ | ------ | ------ | ----- |
| Creating users | 1        | 1      | 20       | 20     | -      | 340 μs | -      | 6 ms  |
| Write posts    | 10       | 100    | 1        | 1 000  | 459 μs | 9 ms   | 304 ms | 67 s  |
| Write comments | 10       | 100    | 1        | 1 000  | 431 μs | 15 ms  | 99 ms  | 65 s  |
| Read posts     | 100      | 100    | 10       | 10 000 | 384 μs | 23 ms  | 351 ms | 68 s  |
| Read comments  | 100      | 100    | 10       | 10 000 | 61 μs  | 24 ms  | 213 ms | 68 s  |
| Database size  | 1 627 kB | 785 kB |          |        |        |        |        |       |

Running purely off a file significantly decreases performance. While the minimum write times remain expectedly the same as with memory mapped option (that uses the same underlying persistent file storage for writes) the average and particularly maximum times increased dramatically. This indicates that for data sets too large to fit to RAM running purely off a file is not a viable option either due to prohibitively bad performance. Therefore, a different strategy would be required (in-memory caching, splitting the data set over multiple databases etc.).

The file based database might be suitable for write heavy use cases with huge amounts of data such as log store where operations can be serialized to limit the contention and reads/searches are relatively infrequent and do not collide with writes often.

### In memory (cache only)

The benchmark run with [default settings](#default-settings) using in-memory cache only (no persistence):

**Windows**

| Description    | Threads  | Iters  | Per iter | Count  | Min   | Avg    | Max    | Total  |
| -------------- | -------- | ------ | -------- | ------ | ----- | ------ | ------ | ------ |
| Creating users | 1        | 1      | 20       | 20     | -     | 9 μs   | -      | 189 μs |
| Write posts    | 10       | 100    | 1        | 1 000  | 11 μs | 5 ms   | 442 ms | 3 s    |
| Write comments | 10       | 100    | 1        | 1 000  | 10 μs | 10 ms  | 440 ms | 3 s    |
| Read posts     | 100      | 100    | 10       | 10 000 | 14 μs | 300 μs | 7 ms   | 6 s    |
| Read comments  | 100      | 100    | 10       | 10 000 | 13 μs | 319 μs | 7 ms   | 6 s    |
| Database size  | 1 627 kB | 785 kB |          |        |       |        |        |        |

**Debian (Hyper-V/WSL2)**

| Description    | Threads  | Iters  | Per iter | Count  | Min  | Avg    | Max  | Total  |
| -------------- | -------- | ------ | -------- | ------ | ---- | ------ | ---- | ------ |
| Creating users | 1        | 1      | 20       | 20     | -    | 6 μs   | -    | 125 μs |
| Write posts    | 10       | 100    | 1        | 1 000  | 6 μs | 503 μs | 7 ms | 2 s    |
| Write comments | 10       | 100    | 1        | 1 000  | 4 μs | 374 μs | 1 ms | 1 s    |
| Read posts     | 100      | 100    | 10       | 10 000 | 9 μs | 729 μs | 8 ms | 5 s    |
| Read comments  | 100      | 100    | 10       | 10 000 | 6 μs | 741 μs | 5 ms | 5 s    |
| Database size  | 1 627 kB | 785 kB |          |        |      |        |      |        |

Unsurprisingly by far the fastest option. Operating purely off RAM offers unmatched performance but without any persistence such a database can be of limited use. Still for caching purposes this solution is very viable offering sub-millisecond performance for all operations (read & write) with minimal impact of contention even in highly contested cases such as the one being benchmarked.

### In Memory (10x)

Increasing the number of writers and readers 10x:

| Description    | Threads | Iters    | Per iter | Count   | Min   | Avg    | Max    | Total |
| -------------- | ------- | -------- | -------- | ------- | ----- | ------ | ------ | ----- |
| Creating users | 1       | 1        | 200      | 200     | -     | 9 μs   | -      | 1 ms  |
| Write posts    | 100     | 100      | 1        | 10 000  | 10 μs | 152 ms | 34 s   | 497 s |
| Write comments | 100     | 100      | 1        | 10 000  | 9 μs  | 154 ms | 64 s   | 496 s |
| Read posts     | 1 000   | 100      | 10       | 100 000 | 21 μs | 3 ms   | 235 ms | 497 s |
| Read comments  | 1 000   | 100      | 10       | 100 000 | 12 μs | 4 ms   | 344 ms | 497 s |
| Database size  | 12 MB   | 6 528 kB |          |         |       |        |        |       |

## Flamegraph

The following is the ["flamegraph"](https://github.com/flamegraph-rs/flamegraph) illustrating what the benchmark is spending most time on. The obvious answer (as predicted) is the async orchestration through tokio followed by the database running queries. Digging down the callgraph there is no immediate performance bottleneck (such as memory allocation) that could be significantly optimized. The database functionality seems to be evenly distributed matching the expectations given what is being run:

![Flamegraph](/images/flamegraph.svg)

## Conclusion

The used benchmark simulates highly contested database environment where dozens of writers and hundreds of readers are using the database at the same time. Tweaking the values (e.g. increasing/decreasing) the writers/readers had no significant effect on overall results meaning that the database can scale, and the principles hold under all circumstances. Running the benchmark in 2 OSs with 3 different storage backends showed that results will vary depending primarily on use (or not) of RAM for caching and on the level of data contention. The contention slowdown however is by no means linear is largely down to the scheduling of tasks (tasks not actually being executed and waiting their turn) - more powerful hardware would improve the results significantly (vertical scalability). As demonstrated `agdb` can handle even an extreme load such as the one in the benchmark. The flamegraph has also shown that the database itself is well optimized and there are no obvious/easy wins with most of the time being taken by orchestration (Tokio runtime) as expected. When using `agdb` your bottlenecks will likely lay elsewhere and not in the database itself.

Some advice:

-   Always measure your use case but do not rely on micro-benchmarks, use realistic workloads. See `Creating users` line in each table which is equivalent to an isolated micro-benchmark and compare it with the rest of the table that demonstrates realistic load with contention.
-   Correct storage backend matters. While the default is usually the best choice offering persistence and speed it comes with certain caveats:
    -   Do not use memory mapped database if you store terabytes of data or your data set is likely to exceed your available RAM size.
    -   Do not use memory mapped database if your use case is write heavy with infrequent reads. The memory mapping aids only in reading and slows down the writes a little bit.
    -   Do not use in-memory cache if you need persistence even though it is the fastest.
