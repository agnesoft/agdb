// GENERATED. DO NOT MODIFY AS ANY CHANGES WILL BE LOST.

//query_test_generator.js

import { describe, expect, it } from "vitest";
import test_queries from "../../../agdb_server/openapi/test_queries.json";
import {
    QueryBuilder,
    CountComparison,
    Comparison,
    DbKeyOrder,
} from "../src/index";

class T {
    db_id: undefined = undefined;
    value1: string = "";
    value2: number = 0;
}

describe("query tests", () => {
    it(`QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("age".into()),DbKeyOrder::Asc("name".into())]).query()`, () => {
        let query = `QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("age".into()),DbKeyOrder::Asc("name".into())]).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("age"), DbKeyOrder.Asc("name")])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().edge().query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().edge().query()`;
        let builder = QueryBuilder.search().from(1).where().edge().query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .distance(CountComparison.LessThan(3))
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().key("k").value(Comparison::Equal(1.into())).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().key("k").value(Comparison::Equal(1.into())).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .key("k")
            .value(Comparison.Equal(1))
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`, () => {
        let query = `QueryBuilder::insert().nodes().values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.search().from(1).where().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = `QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query()`;
        let builder = QueryBuilder.select()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().values(vec!["k1".into(),"k2".into()]).ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = `QueryBuilder::remove().values(vec!["k1".into(),"k2".into()]).ids(QueryBuilder::search().from("a").query()).query()`;
        let builder = QueryBuilder.remove()
            .values(["k1", "k2"])
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = `QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query()`;
        let builder = QueryBuilder.select()
            .key_count()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`, () => {
        let query = `QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from(QueryBuilder.search().from("a").where().node().query())
            .to(QueryBuilder.search().from("b").where().node().query())
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from("a").query()`, () => {
        let query = `QueryBuilder::search().from("a").query()`;
        let builder = QueryBuilder.search().from("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().index("age").value(20).query()`, () => {
        let query = `QueryBuilder::search().index("age").value(20).query()`;
        let builder = QueryBuilder.search().index("age").value(20).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().beyond().keys(vec!["k".into()]).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().beyond().keys(vec!["k".into()]).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .beyond()
            .keys(["k"])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(1.into())).end_where().query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(1.into())).end_where().query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .node()
            .or()
            .where()
            .edge()
            .and()
            .key("k")
            .value(Comparison.Contains(1))
            .end_where()
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().ids("a").query()`, () => {
        let query = `QueryBuilder::remove().ids("a").query()`;
        let builder = QueryBuilder.remove().ids("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().key_count().ids("a").query()`, () => {
        let query = `QueryBuilder::select().key_count().ids("a").query()`;
        let builder = QueryBuilder.select().key_count().ids("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().not().keys(vec!["k1".into(),"k2".into()]).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().not().keys(vec!["k1".into(),"k2".into()]).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .not()
            .keys(["k1", "k2"])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().depth_first().from("a").query()`, () => {
        let query = `QueryBuilder::search().depth_first().from("a").query()`;
        let builder = QueryBuilder.search().depth_first().from("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from("a").to(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::insert().edges().from("a").to(vec![1,2]).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from("a")
            .to([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().ids(1).count(1).query()`, () => {
        let query = `QueryBuilder::insert().nodes().ids(1).count(1).query()`;
        let builder = QueryBuilder.insert().nodes().ids(1).count(1).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().ids(vec![1,2]).count(1).query()`, () => {
        let query = `QueryBuilder::insert().nodes().ids(vec![1,2]).count(1).query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .ids([1, 2])
            .count(1)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().values(vec![vec![("k","v").into(),(1,10).into()],vec![("k",2).into()]]).ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = `QueryBuilder::insert().values(vec![vec![("k","v").into(),(1,10).into()],vec![("k",2).into()]]).ids(QueryBuilder::search().from("a").query()).query()`;
        let builder = QueryBuilder.insert()
            .values([
                [
                    ["k", "v"],
                    [1, 10],
                ],
                [["k", 2]],
            ])
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = `QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query()`;
        let builder = QueryBuilder.select()
            .keys()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().key_count().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().key_count().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select().key_count().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from("a").to(vec![1,2]).values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`, () => {
        let query = `QueryBuilder::insert().edges().from("a").to(vec![1,2]).values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from("a")
            .to([1, 2])
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().count(2).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`, () => {
        let query = `QueryBuilder::insert().nodes().count(2).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .count(2)
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from(vec![1,2]).to(vec![2,3]).each().query()`, () => {
        let query = `QueryBuilder::insert().edges().from(vec![1,2]).to(vec![2,3]).each().query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from([1, 2])
            .to([2, 3])
            .each()
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().ids(vec![-3,-4]).from(1).to(2).query()`, () => {
        let query = `QueryBuilder::insert().edges().ids(vec![-3,-4]).from(1).to(2).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .ids([-3, -4])
            .from(1)
            .to(2)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().values(vec!["k1".into(),"k2".into()]).ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::remove().values(vec!["k1".into(),"k2".into()]).ids(vec![1,2]).query()`;
        let builder = QueryBuilder.remove()
            .values(["k1", "k2"])
            .ids([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().edge_count_from().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().edge_count_from().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select()
            .edge_count_from()
            .ids([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().index("key").query()`, () => {
        let query = `QueryBuilder::insert().index("key").query()`;
        let builder = QueryBuilder.insert().index("key").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from("a").to(vec![1,2]).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`, () => {
        let query = `QueryBuilder::insert().edges().from("a").to(vec![1,2]).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from("a")
            .to([1, 2])
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().aliases(vec!["a","b"]).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`, () => {
        let query = `QueryBuilder::insert().nodes().aliases(vec!["a","b"]).values_uniform(vec![("k","v").into(),(1,10).into()]).query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .aliases(["a", "b"])
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from(vec![1,2]).to(vec![2,3]).query()`, () => {
        let query = `QueryBuilder::insert().edges().from(vec![1,2]).to(vec![2,3]).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from([1, 2])
            .to([2, 3])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().aliases(vec!["a","b"]).query()`, () => {
        let query = `QueryBuilder::insert().nodes().aliases(vec!["a","b"]).query()`;
        let builder = QueryBuilder.insert().nodes().aliases(["a", "b"]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().values_uniform(vec![("k","v").into(),(1,10).into()]).ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = `QueryBuilder::insert().values_uniform(vec![("k","v").into(),(1,10).into()]).ids(QueryBuilder::search().from("a").query()).query()`;
        let builder = QueryBuilder.insert()
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().aliases("a").query()`, () => {
        let query = `QueryBuilder::remove().aliases("a").query()`;
        let builder = QueryBuilder.remove().aliases("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from(1).to(2).query()`, () => {
        let query = `QueryBuilder::insert().edges().from(1).to(2).query()`;
        let builder = QueryBuilder.insert().edges().from(1).to(2).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().ids(vec!["a","b"]).count(1).query()`, () => {
        let query = `QueryBuilder::insert().nodes().ids(vec!["a","b"]).count(1).query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .ids(["a", "b"])
            .count(1)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().node().and().distance(CountComparison::GreaterThanOrEqual(3)).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().node().and().distance(CountComparison::GreaterThanOrEqual(3)).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .node()
            .and()
            .distance(CountComparison.GreaterThanOrEqual(3))
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().values(vec![vec![("k","v").into(),(1,10).into()],vec![("k",2).into()]]).ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::insert().values(vec![vec![("k","v").into(),(1,10).into()],vec![("k",2).into()]]).ids(vec![1,2]).query()`;
        let builder = QueryBuilder.insert()
            .values([
                [
                    ["k", "v"],
                    [1, 10],
                ],
                [["k", 2]],
            ])
            .ids([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().index("key").query()`, () => {
        let query = `QueryBuilder::remove().index("key").query()`;
        let builder = QueryBuilder.remove().index("key").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().aliases().query()`, () => {
        let query = `QueryBuilder::select().aliases().query()`;
        let builder = QueryBuilder.select().aliases().query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().ids("a").query()`, () => {
        let query = `QueryBuilder::select().ids("a").query()`;
        let builder = QueryBuilder.select().ids("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().edge().query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().node().or().edge().query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .node()
            .or()
            .edge()
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().node().query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().node().query()`;
        let builder = QueryBuilder.search().from(1).where().node().query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Equal(1.into())).end_where().query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Equal(1.into())).end_where().query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .node()
            .or()
            .where()
            .edge()
            .and()
            .key("k")
            .value(Comparison.Equal(1))
            .end_where()
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`, () => {
        let query = `QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from(QueryBuilder.search().from("a").where().node().query())
            .to(QueryBuilder.search().from("b").where().node().query())
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().values_uniform(vec![("k","v").into(),(1,10).into()]).ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::insert().values_uniform(vec![("k","v").into(),(1,10).into()]).ids(vec![1,2]).query()`;
        let builder = QueryBuilder.insert()
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .ids([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().keys().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().keys().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select().keys().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().edge_count_to().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().edge_count_to().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select().edge_count_to().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().element(&T::default()).query()`, () => {
        let query = `QueryBuilder::insert().element(&T::default()).query()`;
        let builder = QueryBuilder.insert().element(new T()).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().ids(-3).from(1).to(2).query()`, () => {
        let query = `QueryBuilder::insert().edges().ids(-3).from(1).to(2).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .ids(-3)
            .from(1)
            .to(2)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1)).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1)).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count_from(CountComparison.Equal(1))
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().indexes().query()`, () => {
        let query = `QueryBuilder::select().indexes().query()`;
        let builder = QueryBuilder.select().indexes().query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().count(2).query()`, () => {
        let query = `QueryBuilder::insert().nodes().count(2).query()`;
        let builder = QueryBuilder.insert().nodes().count(2).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).limit(5).query()`, () => {
        let query = `QueryBuilder::search().from(1).limit(5).query()`;
        let builder = QueryBuilder.search().from(1).limit(5).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).limit(5).query()`, () => {
        let query = `QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).limit(5).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("k")])
            .limit(5)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query()).from(1).to(2).query()`, () => {
        let query = `QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query()).from(1).to(2).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .ids(QueryBuilder.search().from(1).where().edge().query())
            .from(1)
            .to(2)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).offset(10).limit(5).query()`, () => {
        let query = `QueryBuilder::search().from(1).offset(10).limit(5).query()`;
        let builder = QueryBuilder.search().from(1).offset(10).limit(5).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().ids("a").count(1).query()`, () => {
        let query = `QueryBuilder::insert().nodes().ids("a").count(1).query()`;
        let builder = QueryBuilder.insert().nodes().ids("a").count(1).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().aliases("a").ids("b").query()`, () => {
        let query = `QueryBuilder::insert().aliases("a").ids("b").query()`;
        let builder = QueryBuilder.insert().aliases("a").ids("b").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().elements(&[T::default(),T::default()]).query()`, () => {
        let query = `QueryBuilder::insert().elements(&[T::default(),T::default()]).query()`;
        let builder = QueryBuilder.insert()
            .elements([new T(), new T()])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2)).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2)).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count(CountComparison.GreaterThan(2))
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1)).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1)).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count_to(CountComparison.NotEqual(1))
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains((vec![1,2]).into())).end_where().query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains((vec![1,2]).into())).end_where().query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .node()
            .or()
            .where()
            .edge()
            .and()
            .key("k")
            .value(Comparison.Contains([1, 2]))
            .end_where()
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().ids(vec!["a","b"]).query()`, () => {
        let query = `QueryBuilder::remove().ids(vec!["a","b"]).query()`;
        let builder = QueryBuilder.remove().ids(["a", "b"]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().values(vec!["k".into(),"k2".into()]).ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().values(vec!["k".into(),"k2".into()]).ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select()
            .values(["k", "k2"])
            .ids([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from("a").to("b").query()`, () => {
        let query = `QueryBuilder::insert().edges().from("a").to("b").query()`;
        let builder = QueryBuilder.insert().edges().from("a").to("b").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().to(1).query()`, () => {
        let query = `QueryBuilder::search().to(1).query()`;
        let builder = QueryBuilder.search().to(1).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().elements().query()`, () => {
        let query = `QueryBuilder::search().elements().query()`;
        let builder = QueryBuilder.search().elements().query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().not().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().not().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .not()
            .ids([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().node_count().query()`, () => {
        let query = `QueryBuilder::select().node_count().query()`;
        let builder = QueryBuilder.select().node_count().query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = `QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query()`;
        let builder = QueryBuilder.select()
            .aliases()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().keys(vec!["k1".into(),"k2".into()]).query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().keys(vec!["k1".into(),"k2".into()]).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .keys(["k1", "k2"])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().values(vec!["k".into(),"k2".into()]).ids("a").query()`, () => {
        let query = `QueryBuilder::select().values(vec!["k".into(),"k2".into()]).ids("a").query()`;
        let builder = QueryBuilder.select()
            .values(["k", "k2"])
            .ids("a")
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query()`, () => {
        let query = `QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("k")])
            .offset(10)
            .limit(5)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().breadth_first().from("a").query()`, () => {
        let query = `QueryBuilder::search().breadth_first().from("a").query()`;
        let builder = QueryBuilder.search().breadth_first().from("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = `QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query()`;
        let builder = QueryBuilder.remove()
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query()`, () => {
        let query = `QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query()`;
        let builder = QueryBuilder.insert()
            .edges()
            .from(QueryBuilder.search().from("a").where().node().query())
            .to(QueryBuilder.search().from("b").where().node().query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().edge_count().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().edge_count().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select().edge_count().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().aliases().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::select().aliases().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.select().aliases().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::remove().ids(vec![1,2]).query()`;
        let builder = QueryBuilder.remove().ids([1, 2]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().aliases(vec!["a","b"]).query()`, () => {
        let query = `QueryBuilder::remove().aliases(vec!["a","b"]).query()`;
        let builder = QueryBuilder.remove().aliases(["a", "b"]).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().values(vec!["k".into(),"k2".into()]).ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = `QueryBuilder::select().values(vec!["k".into(),"k2".into()]).ids(QueryBuilder::search().from(1).query()).query()`;
        let builder = QueryBuilder.select()
            .values(["k", "k2"])
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().aliases(vec!["a","b"]).ids(vec![1,2]).query()`, () => {
        let query = `QueryBuilder::insert().aliases(vec!["a","b"]).ids(vec![1,2]).query()`;
        let builder = QueryBuilder.insert()
            .aliases(["a", "b"])
            .ids([1, 2])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::select().keys().ids("a").query()`, () => {
        let query = `QueryBuilder::select().keys().ids("a").query()`;
        let builder = QueryBuilder.select().keys().ids("a").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query()).count(1).query()`, () => {
        let query = `QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query()).count(1).query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .ids(QueryBuilder.search().from(1).query())
            .count(1)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).where_().not_beyond().ids("a").query()`, () => {
        let query = `QueryBuilder::search().from(1).where_().not_beyond().ids("a").query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .where()
            .not_beyond()
            .ids("a")
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().aliases(vec!["a","b"]).values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`, () => {
        let query = `QueryBuilder::insert().nodes().aliases(vec!["a","b"]).values(vec![vec![("k",1).into()],vec![("k",2).into()]]).query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .aliases(["a", "b"])
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::remove().ids(1).query()`, () => {
        let query = `QueryBuilder::remove().ids(1).query()`;
        let builder = QueryBuilder.remove().ids(1).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().nodes().ids("a").aliases("a").query()`, () => {
        let query = `QueryBuilder::insert().nodes().ids("a").aliases("a").query()`;
        let builder = QueryBuilder.insert()
            .nodes()
            .ids("a")
            .aliases("a")
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from("a").to("b").query()`, () => {
        let query = `QueryBuilder::search().from("a").to("b").query()`;
        let builder = QueryBuilder.search().from("a").to("b").query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).offset(10).query()`, () => {
        let query = `QueryBuilder::search().from(1).offset(10).query()`;
        let builder = QueryBuilder.search().from(1).offset(10).query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).query()`, () => {
        let query = `QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).query()`;
        let builder = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("k")])
            .offset(10)
            .query();
        expect(builder).toEqual(test_queries[query]);
    });

    it(`QueryBuilder::insert().aliases("a").ids(1).query()`, () => {
        let query = `QueryBuilder::insert().aliases("a").ids(1).query()`;
        let builder = QueryBuilder.insert().aliases("a").ids(1).query();
        expect(builder).toEqual(test_queries[query]);
    });
});
