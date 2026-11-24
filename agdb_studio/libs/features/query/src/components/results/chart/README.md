# Charts

The **Chart** view draws query results as a bar chart, line chart, donut or
stat tile. It sits next to Graph, Table and JSON on the query page, in the
dashboard widget editor and in dashboard widgets. The chart settings are saved
with the query, and with the widget when it is pinned to a dashboard.

## How a chart is built

Every chart is a **measure** of groups of elements:

| Setting   | Choices                                                                                  |
| --------- | ---------------------------------------------------------------------------------------- |
| **Show**  | `Count` of elements, or `Sum` / `Average` / `Min` / `Max` of a numeric property (`of …`) |
| **by**    | a property (one group per value), or `Query of the batch` (one group per query)          |
| **Lines** | line charts of a batch only: `One`, or `One per query` (the "by" property runs along X)  |

- **Bar:** one bar per group, largest first. At most 12; the rest fold into a
  grey "Other".
- **Line:** groups along X, ordered as numbers, then dates, then text. A line
  needs an ordered X, so it is not offered when grouping by query.
- **Donut:** shares of a total, so only `Count` and `Sum`. At most 5 slices,
  then "Other".
- **Stat:** the measure of all elements as one number.

Grouped by a property, an element returned by several queries counts once.
Grouped by query, each query's count is its result number, so counting queries
such as `node_count` chart too.

The first time you open Chart, it suggests a chart that fits the results:

- a bar per query for a batch of counts;
- a stat tile for a single count;
- otherwise a count by the first text property that has between 2 and 12
  values, shared by at least two elements.

Charts read values from the results themselves. Use `select().search()…`
rather than a bare `search()`, which returns ids only.

## Pasting the examples

Copy a block and paste it into a query row (or press Ctrl+V in the step
input). Each `QueryBuilder::` line becomes its own row, and all rows run
together as one batch. Run inserts on the **exec_mut** tab and reads on
**exec**. Use a fresh database, because the examples create nodes with fixed
aliases (`shop`, `p1`, `cpu1`, …).

## Example 1: a shop

Twelve products with a category, a price, a rating and the date they were
added. The rating of the kettle is `"n/a"`, so it is not a number.

```rust
QueryBuilder::insert().nodes().aliases("shop").query()
QueryBuilder::insert().nodes().aliases("p1").values_uniform([("kind", "product").into(), ("name", "Desk").into(), ("category", "furniture").into(), ("price", 250).into(), ("rating", 4.5).into(), ("added", "2026-01-12").into()]).query()
QueryBuilder::insert().nodes().aliases("p2").values_uniform([("kind", "product").into(), ("name", "Chair").into(), ("category", "furniture").into(), ("price", 120).into(), ("rating", 4.1).into(), ("added", "2026-02-03").into()]).query()
QueryBuilder::insert().nodes().aliases("p3").values_uniform([("kind", "product").into(), ("name", "Shelf").into(), ("category", "furniture").into(), ("price", 80).into(), ("rating", 3.7).into(), ("added", "2026-03-21").into()]).query()
QueryBuilder::insert().nodes().aliases("p4").values_uniform([("kind", "product").into(), ("name", "Laptop").into(), ("category", "electronics").into(), ("price", 1400).into(), ("rating", 4.7).into(), ("added", "2026-01-30").into()]).query()
QueryBuilder::insert().nodes().aliases("p5").values_uniform([("kind", "product").into(), ("name", "Monitor").into(), ("category", "electronics").into(), ("price", 320).into(), ("rating", 4.3).into(), ("added", "2026-04-08").into()]).query()
QueryBuilder::insert().nodes().aliases("p6").values_uniform([("kind", "product").into(), ("name", "Keyboard").into(), ("category", "electronics").into(), ("price", 90).into(), ("rating", 4.0).into(), ("added", "2026-05-17").into()]).query()
QueryBuilder::insert().nodes().aliases("p7").values_uniform([("kind", "product").into(), ("name", "Mouse").into(), ("category", "electronics").into(), ("price", 40).into(), ("rating", 3.9).into(), ("added", "2026-06-02").into()]).query()
QueryBuilder::insert().nodes().aliases("p8").values_uniform([("kind", "product").into(), ("name", "Novel").into(), ("category", "books").into(), ("price", 18).into(), ("rating", 4.4).into(), ("added", "2026-02-14").into()]).query()
QueryBuilder::insert().nodes().aliases("p9").values_uniform([("kind", "product").into(), ("name", "Cookbook").into(), ("category", "books").into(), ("price", 32).into(), ("rating", 4.6).into(), ("added", "2026-07-09").into()]).query()
QueryBuilder::insert().nodes().aliases("p10").values_uniform([("kind", "product").into(), ("name", "Lamp").into(), ("category", "lighting").into(), ("price", 60).into(), ("rating", 3.5).into(), ("added", "2026-08-25").into()]).query()
QueryBuilder::insert().nodes().aliases("p11").values_uniform([("kind", "product").into(), ("name", "Mug").into(), ("category", "kitchen").into(), ("price", 12).into(), ("rating", 4.8).into(), ("added", "2026-09-11").into()]).query()
QueryBuilder::insert().nodes().aliases("p12").values_uniform([("kind", "product").into(), ("name", "Kettle").into(), ("category", "kitchen").into(), ("price", 45).into(), ("rating", "n/a").into(), ("added", "2026-10-04").into()]).query()
QueryBuilder::insert().edges().from("shop").to(["p1", "p2", "p3", "p4", "p5", "p6", "p7", "p8", "p9", "p10", "p11", "p12"]).query()
```

Then run all products and open **Chart**:

```rust
QueryBuilder::select().search().from("shop").where_().key("kind").value("product").query()
```

| Settings                           | Expected                                                                               |
| ---------------------------------- | -------------------------------------------------------------------------------------- |
| as suggested                       | Bar, **Count by category**: electronics 4, furniture 3, books 2, kitchen 2, lighting 1 |
| Bar, Sum of price by category      | electronics 1.9K, furniture 450, lighting 60, kitchen 57, books 50                     |
| Donut, Sum of price by category    | 5 slices, 2.5K in the middle                                                           |
| Donut, Count by name               | 4 slices and "Other"; footnote `8 smaller groups in "Other"`                           |
| Bar, Average of rating by category | footnote `1 element without "category" or "rating" left out` (the kettle)              |
| Line, Sum of price by added        | one line along dates, January to October                                               |
| Stat, Sum of price                 | 2,467                                                                                  |
| Stat, Average of rating            | 4.23                                                                                   |

Bars, the donut's total and the axes show rounded numbers (1.9K); hover over a
mark, or focus the chart and use the arrow keys, for the exact value.

## Example 2: comparing queries

With the shop data, one query per category. In a new tab, the suggestion is
Count by category again, as the elements have one. Set **by** to **Query of the
batch** for a bar per query, in batch order: books 2, electronics 4,
furniture 3.

```rust
QueryBuilder::select().search().from("shop").where_().key("category").value("books").query()
QueryBuilder::select().search().from("shop").where_().key("category").value("electronics").query()
QueryBuilder::select().search().from("shop").where_().key("category").value("furniture").query()
```

Add three more queries and switch to **Donut**. The six queries no longer fit, so the largest come first and the smallest fold into "Other", with a
footnote such as `2 smaller queries in "Other"`.

```rust
QueryBuilder::select().search().from("shop").where_().key("category").value("lighting").query()
QueryBuilder::select().search().from("shop").where_().key("category").value("kitchen").query()
QueryBuilder::select().search().from("shop").where_().key("price").greater_than(100).query()
```

Queries that return counts rather than elements chart as well. This batch
gives one bar per query:

```rust
QueryBuilder::select().node_count().query()
QueryBuilder::select().edge_count().query()
```

Either line on its own gives a stat tile.

## Example 3: trends, one line per query

Seven monthly readings for three metrics. The first three lines create the
readings; the rest give each reading its month, then its value.

```rust
QueryBuilder::insert().nodes().aliases(["cpu1", "cpu2", "cpu3", "cpu4", "cpu5", "cpu6", "cpu7"]).values_uniform([("kind", "reading").into(), ("metric", "cpu").into()]).query()
QueryBuilder::insert().nodes().aliases(["memory1", "memory2", "memory3", "memory4", "memory5", "memory6", "memory7"]).values_uniform([("kind", "reading").into(), ("metric", "memory").into()]).query()
QueryBuilder::insert().nodes().aliases(["disk1", "disk2", "disk3", "disk4", "disk5", "disk6", "disk7"]).values_uniform([("kind", "reading").into(), ("metric", "disk").into()]).query()
QueryBuilder::insert().values([[("month", "2026-05").into()], [("month", "2026-06").into()], [("month", "2026-07").into()], [("month", "2026-08").into()], [("month", "2026-09").into()], [("month", "2026-10").into()], [("month", "2026-11").into()]]).ids(["cpu1", "cpu2", "cpu3", "cpu4", "cpu5", "cpu6", "cpu7"]).query()
QueryBuilder::insert().values([[("value", 40).into()], [("value", 56).into()], [("value", 60).into()], [("value", 62).into()], [("value", 65).into()], [("value", 68).into()], [("value", 69).into()]]).ids(["cpu1", "cpu2", "cpu3", "cpu4", "cpu5", "cpu6", "cpu7"]).query()
QueryBuilder::insert().values([[("month", "2026-05").into()], [("month", "2026-06").into()], [("month", "2026-07").into()], [("month", "2026-08").into()], [("month", "2026-09").into()], [("month", "2026-10").into()], [("month", "2026-11").into()]]).ids(["memory1", "memory2", "memory3", "memory4", "memory5", "memory6", "memory7"]).query()
QueryBuilder::insert().values([[("value", 85).into()], [("value", 73).into()], [("value", 67).into()], [("value", 62).into()], [("value", 58).into()], [("value", 52).into()], [("value", 51).into()]]).ids(["memory1", "memory2", "memory3", "memory4", "memory5", "memory6", "memory7"]).query()
QueryBuilder::insert().values([[("month", "2026-05").into()], [("month", "2026-06").into()], [("month", "2026-07").into()], [("month", "2026-08").into()], [("month", "2026-09").into()], [("month", "2026-10").into()], [("month", "2026-11").into()]]).ids(["disk1", "disk2", "disk3", "disk4", "disk5", "disk6", "disk7"]).query()
QueryBuilder::insert().values([[("value", 6).into()], [("value", 11).into()], [("value", 14).into()], [("value", 16).into()], [("value", 18).into()], [("value", 21).into()], [("value", 26).into()]]).ids(["disk1", "disk2", "disk3", "disk4", "disk5", "disk6", "disk7"]).query()
```

Read one metric per query:

```rust
QueryBuilder::select().ids(["cpu1", "cpu2", "cpu3", "cpu4", "cpu5", "cpu6", "cpu7"]).query()
QueryBuilder::select().ids(["memory1", "memory2", "memory3", "memory4", "memory5", "memory6", "memory7"]).query()
QueryBuilder::select().ids(["disk1", "disk2", "disk3", "disk4", "disk5", "disk6", "disk7"]).query()
```

Set **Line**, Show **Sum** of **value**, by **month**, Lines **One per
query**. There is one reading per month, so Average gives the same lines. You
should see three lines along a date axis, 2026-05 to 2026-11:

- **#1, CPU:** 40 to 69, rising.
- **#2, memory:** 85 to 51, falling, crossing CPU at 2026-08 (62 each).
- **#3, disk:** 6 to 26, slow growth.

Each line keeps the colour its query gets in the graph's **Colours →
Each query**. The legend names the queries, and up to four lines are also
named at their right end. Hover over the chart, or focus it and use the arrow
keys, to see every query's value for one month. A query without a value at
some X shows `–` there, and its line passes over that X. From six queries on,
the fifth and later are drawn as one grey "Other" line.

## Example 4: when there is nothing to chart

The chart says what is missing instead of drawing an empty plot. With the shop
data, run each of these on its own. Rows 1 and 3 replace the query in the tab
where Example 1 was charted, so its saved chart meets results that do not fit
it; row 2 runs in a new tab.

| Query                                                                                                                      | Expected                                                                                                                                            |
| -------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `QueryBuilder::select().ids("shop").query()`                                                                               | A node without properties. The saved Bar by category says `No element has "category"…`, and Sum / Average / Min / Max are greyed out.               |
| `QueryBuilder::select().ids(["p1", "p2", "p3"]).query()`                                                                   | Three furniture products with unique names: nothing worth grouping, so a stat tile of 3. Bar by category is a single bar.                           |
| `QueryBuilder::select().values(["category", "price"]).search().from("shop").where_().key("kind").value("product").query()` | Only two properties come back. With the chart saved as Count by name, it says `No element has "name"…`; pick **category** in **by** to chart again. |

Opening Chart before the first run shows `Nothing to chart yet — run the
query.` A grouping where every element has its own value, such as Count by
price, is drawn, but each group counts 1.

## On a dashboard

Pin a query with the Chart view to a dashboard, or pick Chart in the widget
editor. On the dashboard, the settings sit behind the sliders button in the
widget's corner. A change there raises **Unsaved view**: **Save** keeps it in
the widget and **Reset** goes back to the saved chart.
