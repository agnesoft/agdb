// GENERATED. DO NOT MODIFY AS ANY CHANGES WILL BE LOST.

//query_test_generator.js

import { describe, expect, it } from "vitest";
import test_queries from "../../../agdb_server/test_queries.json";
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
    it(`QueryBuilder::insert().aliases("a").ids(1).query()`, () => {
        let query = QueryBuilder.insert().aliases("a").ids(1).query();
        expect(query).toEqual(test_queries[0][1]);
    });

    it(`QueryBuilder::insert().aliases("a").ids("b").query()`, () => {
        let query = QueryBuilder.insert().aliases("a").ids("b").query();
        expect(query).toEqual(test_queries[1][1]);
    });

    it(`QueryBuilder::insert().aliases(["a","b"]).ids([1,2]).query()`, () => {
        let query = QueryBuilder.insert()
            .aliases(["a", "b"])
            .ids([1, 2])
            .query();
        expect(query).toEqual(test_queries[2][1]);
    });

    it(`QueryBuilder::insert().edges().from(1).to(2).query()`, () => {
        let query = QueryBuilder.insert().edges().from(1).to(2).query();
        expect(query).toEqual(test_queries[3][1]);
    });

    it(`QueryBuilder::insert().edges().from("a").to("b").query()`, () => {
        let query = QueryBuilder.insert().edges().from("a").to("b").query();
        expect(query).toEqual(test_queries[4][1]);
    });

    it(`QueryBuilder::insert().edges().from("a").to([1,2]).query()`, () => {
        let query = QueryBuilder.insert().edges().from("a").to([1, 2]).query();
        expect(query).toEqual(test_queries[5][1]);
    });

    it(`QueryBuilder::insert().edges().from([1,2]).to([2,3]).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from([1, 2])
            .to([2, 3])
            .query();
        expect(query).toEqual(test_queries[6][1]);
    });

    it(`QueryBuilder::insert().edges().from([1,2]).to([2,3]).each().query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from([1, 2])
            .to([2, 3])
            .each()
            .query();
        expect(query).toEqual(test_queries[7][1]);
    });

    it(`QueryBuilder::insert().edges().from([1,2]).to([2,3]).each().values([[("k",1).into()],[("k",2).into()]]).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from([1, 2])
            .to([2, 3])
            .each()
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(query).toEqual(test_queries[8][1]);
    });

    it(`QueryBuilder::insert().edges().from([1,2]).to([2,3]).each().values_uniform([("k",1).into(),(1,10).into()]).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from([1, 2])
            .to([2, 3])
            .each()
            .values_uniform([
                ["k", 1],
                [1, 10],
            ])
            .query();
        expect(query).toEqual(test_queries[9][1]);
    });

    it(`QueryBuilder::insert().edges().from("a").to([1,2]).values([[("k",1).into()],[("k",2).into()]]).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from("a")
            .to([1, 2])
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(query).toEqual(test_queries[10][1]);
    });

    it(`QueryBuilder::insert().edges().from("a").to([1,2]).values_uniform([("k","v").into(),(1,10).into()]).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from("a")
            .to([1, 2])
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(query).toEqual(test_queries[11][1]);
    });

    it(`QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from(QueryBuilder.search().from("a").where().node().query())
            .to(QueryBuilder.search().from("b").where().node().query())
            .query();
        expect(query).toEqual(test_queries[12][1]);
    });

    it(`QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values([[("k",1).into()],[("k",2).into()]]).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from(QueryBuilder.search().from("a").where().node().query())
            .to(QueryBuilder.search().from("b").where().node().query())
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(query).toEqual(test_queries[13][1]);
    });

    it(`QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform([("k","v").into(),(1,10).into()]).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .from(QueryBuilder.search().from("a").where().node().query())
            .to(QueryBuilder.search().from("b").where().node().query())
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(query).toEqual(test_queries[14][1]);
    });

    it(`QueryBuilder::insert().edges().ids(-3).from(1).to(2).query()`, () => {
        let query = QueryBuilder.insert().edges().ids(-3).from(1).to(2).query();
        expect(query).toEqual(test_queries[15][1]);
    });

    it(`QueryBuilder::insert().edges().ids([-3,-4]).from(1).to(2).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .ids([-3, -4])
            .from(1)
            .to(2)
            .query();
        expect(query).toEqual(test_queries[16][1]);
    });

    it(`QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query()).from(1).to(2).query()`, () => {
        let query = QueryBuilder.insert()
            .edges()
            .ids(QueryBuilder.search().from(1).where().edge().query())
            .from(1)
            .to(2)
            .query();
        expect(query).toEqual(test_queries[17][1]);
    });

    it(`QueryBuilder::insert().index("key").query()`, () => {
        let query = QueryBuilder.insert().index("key").query();
        expect(query).toEqual(test_queries[18][1]);
    });

    it(`QueryBuilder::insert().nodes().count(2).query()`, () => {
        let query = QueryBuilder.insert().nodes().count(2).query();
        expect(query).toEqual(test_queries[19][1]);
    });

    it(`QueryBuilder::insert().nodes().count(2).values_uniform([("k","v").into(),(1,10).into()]).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .count(2)
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(query).toEqual(test_queries[20][1]);
    });

    it(`QueryBuilder::insert().nodes().aliases(["a","b"]).query()`, () => {
        let query = QueryBuilder.insert().nodes().aliases(["a", "b"]).query();
        expect(query).toEqual(test_queries[21][1]);
    });

    it(`QueryBuilder::insert().nodes().aliases(["a","b"]).values([[("k",1).into()],[("k",2).into()]]).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .aliases(["a", "b"])
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(query).toEqual(test_queries[22][1]);
    });

    it(`QueryBuilder::insert().nodes().aliases(["a","b"]).values_uniform([("k","v").into(),(1,10).into()]).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .aliases(["a", "b"])
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(query).toEqual(test_queries[23][1]);
    });

    it(`QueryBuilder::insert().nodes().values([[("k",1).into()],[("k",2).into()]]).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .values([[["k", 1]], [["k", 2]]])
            .query();
        expect(query).toEqual(test_queries[24][1]);
    });

    it(`QueryBuilder::insert().nodes().ids(1).count(1).query()`, () => {
        let query = QueryBuilder.insert().nodes().ids(1).count(1).query();
        expect(query).toEqual(test_queries[25][1]);
    });

    it(`QueryBuilder::insert().nodes().ids([1,2]).count(1).query()`, () => {
        let query = QueryBuilder.insert().nodes().ids([1, 2]).count(1).query();
        expect(query).toEqual(test_queries[26][1]);
    });

    it(`QueryBuilder::insert().nodes().ids("a").count(1).query()`, () => {
        let query = QueryBuilder.insert().nodes().ids("a").count(1).query();
        expect(query).toEqual(test_queries[27][1]);
    });

    it(`QueryBuilder::insert().nodes().ids("a").aliases("a").query()`, () => {
        let query = QueryBuilder.insert().nodes().ids("a").aliases("a").query();
        expect(query).toEqual(test_queries[28][1]);
    });

    it(`QueryBuilder::insert().nodes().ids(["a","b"]).count(1).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .ids(["a", "b"])
            .count(1)
            .query();
        expect(query).toEqual(test_queries[29][1]);
    });

    it(`QueryBuilder::insert().nodes().ids([1,2]).values([[("k","v").into()],[(1,10).into()]]).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .ids([1, 2])
            .values([[["k", "v"]], [[1, 10]]])
            .query();
        expect(query).toEqual(test_queries[30][1]);
    });

    it(`QueryBuilder::insert().nodes().ids([1,2]).values_uniform([("k","v").into(),(1,10).into()]).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .ids([1, 2])
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .query();
        expect(query).toEqual(test_queries[31][1]);
    });

    it(`QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query()).count(1).query()`, () => {
        let query = QueryBuilder.insert()
            .nodes()
            .ids(QueryBuilder.search().from(1).query())
            .count(1)
            .query();
        expect(query).toEqual(test_queries[32][1]);
    });

    it(`QueryBuilder::insert().element(&T::default()).query()`, () => {
        let query = QueryBuilder.insert().element(new T()).query();
        expect(query).toEqual(test_queries[33][1]);
    });

    it(`QueryBuilder::insert().elements(&[T::default(),T::default()]).query()`, () => {
        let query = QueryBuilder.insert().elements([new T(), new T()]).query();
        expect(query).toEqual(test_queries[34][1]);
    });

    it(`QueryBuilder::insert().values([vec![("k","v").into(),(1,10).into()],vec![("k",2).into()]]).ids([1,2]).query()`, () => {
        let query = QueryBuilder.insert()
            .values([
                [
                    ["k", "v"],
                    [1, 10],
                ],
                [["k", 2]],
            ])
            .ids([1, 2])
            .query();
        expect(query).toEqual(test_queries[35][1]);
    });

    it(`QueryBuilder::insert().values([vec![("k","v").into(),(1,10).into()],vec![("k",2).into()]]).ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = QueryBuilder.insert()
            .values([
                [
                    ["k", "v"],
                    [1, 10],
                ],
                [["k", 2]],
            ])
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(query).toEqual(test_queries[36][1]);
    });

    it(`QueryBuilder::insert().values([vec![("k","v").into(),(1,10).into()],vec![("k",2).into()]]).search().from("a").query()`, () => {
        let query = QueryBuilder.insert()
            .values([
                [
                    ["k", "v"],
                    [1, 10],
                ],
                [["k", 2]],
            ])
            .search()
            .from("a")
            .query();
        expect(query).toEqual(test_queries[37][1]);
    });

    it(`QueryBuilder::insert().values_uniform([("k","v").into(),(1,10).into()]).ids([1,2]).query()`, () => {
        let query = QueryBuilder.insert()
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .ids([1, 2])
            .query();
        expect(query).toEqual(test_queries[38][1]);
    });

    it(`QueryBuilder::insert().values_uniform([("k","v").into(),(1,10).into()]).ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = QueryBuilder.insert()
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(query).toEqual(test_queries[39][1]);
    });

    it(`QueryBuilder::insert().values_uniform([("k","v").into(),(1,10).into()]).search().from("a").query()`, () => {
        let query = QueryBuilder.insert()
            .values_uniform([
                ["k", "v"],
                [1, 10],
            ])
            .search()
            .from("a")
            .query();
        expect(query).toEqual(test_queries[40][1]);
    });

    it(`QueryBuilder::remove().aliases("a").query()`, () => {
        let query = QueryBuilder.remove().aliases("a").query();
        expect(query).toEqual(test_queries[41][1]);
    });

    it(`QueryBuilder::remove().aliases(["a","b"]).query()`, () => {
        let query = QueryBuilder.remove().aliases(["a", "b"]).query();
        expect(query).toEqual(test_queries[42][1]);
    });

    it(`QueryBuilder::remove().ids(1).query()`, () => {
        let query = QueryBuilder.remove().ids(1).query();
        expect(query).toEqual(test_queries[43][1]);
    });

    it(`QueryBuilder::remove().ids("a").query()`, () => {
        let query = QueryBuilder.remove().ids("a").query();
        expect(query).toEqual(test_queries[44][1]);
    });

    it(`QueryBuilder::remove().ids([1,2]).query()`, () => {
        let query = QueryBuilder.remove().ids([1, 2]).query();
        expect(query).toEqual(test_queries[45][1]);
    });

    it(`QueryBuilder::remove().ids(["a","b"]).query()`, () => {
        let query = QueryBuilder.remove().ids(["a", "b"]).query();
        expect(query).toEqual(test_queries[46][1]);
    });

    it(`QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = QueryBuilder.remove()
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(query).toEqual(test_queries[47][1]);
    });

    it(`QueryBuilder::remove().search().from("a").query()`, () => {
        let query = QueryBuilder.remove().search().from("a").query();
        expect(query).toEqual(test_queries[48][1]);
    });

    it(`QueryBuilder::remove().index("key").query()`, () => {
        let query = QueryBuilder.remove().index("key").query();
        expect(query).toEqual(test_queries[49][1]);
    });

    it(`QueryBuilder::remove().values(["k1","k2"]).ids([1,2]).query()`, () => {
        let query = QueryBuilder.remove()
            .values(["k1", "k2"])
            .ids([1, 2])
            .query();
        expect(query).toEqual(test_queries[50][1]);
    });

    it(`QueryBuilder::remove().values(["k1","k2"]).ids(QueryBuilder::search().from("a").query()).query()`, () => {
        let query = QueryBuilder.remove()
            .values(["k1", "k2"])
            .ids(QueryBuilder.search().from("a").query())
            .query();
        expect(query).toEqual(test_queries[51][1]);
    });

    it(`QueryBuilder::remove().values(["k1","k2"]).search().from("a").query()`, () => {
        let query = QueryBuilder.remove()
            .values(["k1", "k2"])
            .search()
            .from("a")
            .query();
        expect(query).toEqual(test_queries[52][1]);
    });

    it(`QueryBuilder::select().aliases().ids([1,2]).query()`, () => {
        let query = QueryBuilder.select().aliases().ids([1, 2]).query();
        expect(query).toEqual(test_queries[53][1]);
    });

    it(`QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = QueryBuilder.select()
            .aliases()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(query).toEqual(test_queries[54][1]);
    });

    it(`QueryBuilder::select().aliases().search().from(1).query()`, () => {
        let query = QueryBuilder.select().aliases().search().from(1).query();
        expect(query).toEqual(test_queries[55][1]);
    });

    it(`QueryBuilder::select().aliases().query()`, () => {
        let query = QueryBuilder.select().aliases().query();
        expect(query).toEqual(test_queries[56][1]);
    });

    it(`QueryBuilder::select().edge_count().ids([1,2]).query()`, () => {
        let query = QueryBuilder.select().edge_count().ids([1, 2]).query();
        expect(query).toEqual(test_queries[57][1]);
    });

    it(`QueryBuilder::select().edge_count_from().ids([1,2]).query()`, () => {
        let query = QueryBuilder.select().edge_count_from().ids([1, 2]).query();
        expect(query).toEqual(test_queries[58][1]);
    });

    it(`QueryBuilder::select().edge_count_to().ids([1,2]).query()`, () => {
        let query = QueryBuilder.select().edge_count_to().ids([1, 2]).query();
        expect(query).toEqual(test_queries[59][1]);
    });

    it(`QueryBuilder::select().edge_count().search().from(1).query()`, () => {
        let query = QueryBuilder.select().edge_count().search().from(1).query();
        expect(query).toEqual(test_queries[60][1]);
    });

    it(`QueryBuilder::select().ids("a").query()`, () => {
        let query = QueryBuilder.select().ids("a").query();
        expect(query).toEqual(test_queries[61][1]);
    });

    it(`QueryBuilder::select().ids([1,2]).query()`, () => {
        let query = QueryBuilder.select().ids([1, 2]).query();
        expect(query).toEqual(test_queries[62][1]);
    });

    it(`QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = QueryBuilder.select()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(query).toEqual(test_queries[63][1]);
    });

    it(`QueryBuilder::select().search().from(1).query()`, () => {
        let query = QueryBuilder.select().search().from(1).query();
        expect(query).toEqual(test_queries[64][1]);
    });

    it(`QueryBuilder::select().indexes().query()`, () => {
        let query = QueryBuilder.select().indexes().query();
        expect(query).toEqual(test_queries[65][1]);
    });

    it(`QueryBuilder::select().keys().ids("a").query()`, () => {
        let query = QueryBuilder.select().keys().ids("a").query();
        expect(query).toEqual(test_queries[66][1]);
    });

    it(`QueryBuilder::select().keys().ids([1,2]).query()`, () => {
        let query = QueryBuilder.select().keys().ids([1, 2]).query();
        expect(query).toEqual(test_queries[67][1]);
    });

    it(`QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = QueryBuilder.select()
            .keys()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(query).toEqual(test_queries[68][1]);
    });

    it(`QueryBuilder::select().keys().search().from(1).query()`, () => {
        let query = QueryBuilder.select().keys().search().from(1).query();
        expect(query).toEqual(test_queries[69][1]);
    });

    it(`QueryBuilder::select().key_count().ids("a").query()`, () => {
        let query = QueryBuilder.select().key_count().ids("a").query();
        expect(query).toEqual(test_queries[70][1]);
    });

    it(`QueryBuilder::select().key_count().ids([1,2]).query()`, () => {
        let query = QueryBuilder.select().key_count().ids([1, 2]).query();
        expect(query).toEqual(test_queries[71][1]);
    });

    it(`QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = QueryBuilder.select()
            .key_count()
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(query).toEqual(test_queries[72][1]);
    });

    it(`QueryBuilder::select().key_count().search().from(1).query()`, () => {
        let query = QueryBuilder.select().key_count().search().from(1).query();
        expect(query).toEqual(test_queries[73][1]);
    });

    it(`QueryBuilder::select().node_count().query()`, () => {
        let query = QueryBuilder.select().node_count().query();
        expect(query).toEqual(test_queries[74][1]);
    });

    it(`QueryBuilder::select().values(["k","k2"]).ids("a").query()`, () => {
        let query = QueryBuilder.select().values(["k", "k2"]).ids("a").query();
        expect(query).toEqual(test_queries[75][1]);
    });

    it(`QueryBuilder::select().values(["k","k2"]).ids([1,2]).query()`, () => {
        let query = QueryBuilder.select()
            .values(["k", "k2"])
            .ids([1, 2])
            .query();
        expect(query).toEqual(test_queries[76][1]);
    });

    it(`QueryBuilder::select().values(["k","k2"]).ids(QueryBuilder::search().from(1).query()).query()`, () => {
        let query = QueryBuilder.select()
            .values(["k", "k2"])
            .ids(QueryBuilder.search().from(1).query())
            .query();
        expect(query).toEqual(test_queries[77][1]);
    });

    it(`QueryBuilder::select().values(["k","k2"]).search().from(1).query()`, () => {
        let query = QueryBuilder.select()
            .values(["k", "k2"])
            .search()
            .from(1)
            .query();
        expect(query).toEqual(test_queries[78][1]);
    });

    it(`QueryBuilder::search().from("a").query()`, () => {
        let query = QueryBuilder.search().from("a").query();
        expect(query).toEqual(test_queries[79][1]);
    });

    it(`QueryBuilder::search().to(1).query()`, () => {
        let query = QueryBuilder.search().to(1).query();
        expect(query).toEqual(test_queries[80][1]);
    });

    it(`QueryBuilder::search().from("a").to("b").query()`, () => {
        let query = QueryBuilder.search().from("a").to("b").query();
        expect(query).toEqual(test_queries[81][1]);
    });

    it(`QueryBuilder::search().breadth_first().from("a").query()`, () => {
        let query = QueryBuilder.search().breadth_first().from("a").query();
        expect(query).toEqual(test_queries[82][1]);
    });

    it(`QueryBuilder::search().depth_first().to(1).query()`, () => {
        let query = QueryBuilder.search().depth_first().to(1).query();
        expect(query).toEqual(test_queries[83][1]);
    });

    it(`QueryBuilder::search().depth_first().from("a").query()`, () => {
        let query = QueryBuilder.search().depth_first().from("a").query();
        expect(query).toEqual(test_queries[84][1]);
    });

    it(`QueryBuilder::search().elements().query()`, () => {
        let query = QueryBuilder.search().elements().query();
        expect(query).toEqual(test_queries[85][1]);
    });

    it(`QueryBuilder::search().index("age").value(20).query()`, () => {
        let query = QueryBuilder.search().index("age").value(20).query();
        expect(query).toEqual(test_queries[86][1]);
    });

    it(`QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("age".into()),DbKeyOrder::Asc("name".into())]).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("age"), DbKeyOrder.Asc("name")])
            .query();
        expect(query).toEqual(test_queries[87][1]);
    });

    it(`QueryBuilder::search().from(1).offset(10).query()`, () => {
        let query = QueryBuilder.search().from(1).offset(10).query();
        expect(query).toEqual(test_queries[88][1]);
    });

    it(`QueryBuilder::search().from(1).limit(5).query()`, () => {
        let query = QueryBuilder.search().from(1).limit(5).query();
        expect(query).toEqual(test_queries[89][1]);
    });

    it(`QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("k")])
            .offset(10)
            .query();
        expect(query).toEqual(test_queries[90][1]);
    });

    it(`QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).limit(5).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("k")])
            .limit(5)
            .query();
        expect(query).toEqual(test_queries[91][1]);
    });

    it(`QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Desc("k")])
            .offset(10)
            .limit(5)
            .query();
        expect(query).toEqual(test_queries[92][1]);
    });

    it(`QueryBuilder::search().from(1).offset(10).limit(5).query()`, () => {
        let query = QueryBuilder.search().from(1).offset(10).limit(5).query();
        expect(query).toEqual(test_queries[93][1]);
    });

    it(`QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .distance(CountComparison.LessThan(3))
            .query();
        expect(query).toEqual(test_queries[94][1]);
    });

    it(`QueryBuilder::search().from(1).where_().edge().query()`, () => {
        let query = QueryBuilder.search().from(1).where().edge().query();
        expect(query).toEqual(test_queries[95][1]);
    });

    it(`QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2)).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count(CountComparison.GreaterThan(2))
            .query();
        expect(query).toEqual(test_queries[96][1]);
    });

    it(`QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1)).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count_from(CountComparison.Equal(1))
            .query();
        expect(query).toEqual(test_queries[97][1]);
    });

    it(`QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1)).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count_to(CountComparison.NotEqual(1))
            .query();
        expect(query).toEqual(test_queries[98][1]);
    });

    it(`QueryBuilder::search().from(1).where_().node().query()`, () => {
        let query = QueryBuilder.search().from(1).where().node().query();
        expect(query).toEqual(test_queries[99][1]);
    });

    it(`QueryBuilder::search().from(1).where_().key("k").value(Comparison::Equal(1.into())).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .key("k")
            .value(Comparison.Equal(1))
            .query();
        expect(query).toEqual(test_queries[100][1]);
    });

    it(`QueryBuilder::search().from(1).where_().keys(["k1","k2"]).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .keys(["k1", "k2"])
            .query();
        expect(query).toEqual(test_queries[101][1]);
    });

    it(`QueryBuilder::search().from(1).where_().not().keys(["k1","k2"]).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .not()
            .keys(["k1", "k2"])
            .query();
        expect(query).toEqual(test_queries[102][1]);
    });

    it(`QueryBuilder::search().from(1).where_().ids([1,2]).query()`, () => {
        let query = QueryBuilder.search().from(1).where().ids([1, 2]).query();
        expect(query).toEqual(test_queries[103][1]);
    });

    it(`QueryBuilder::search().from(1).where_().beyond().keys(["k"]).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .beyond()
            .keys(["k"])
            .query();
        expect(query).toEqual(test_queries[104][1]);
    });

    it(`QueryBuilder::search().from(1).where_().not().ids([1,2]).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .not()
            .ids([1, 2])
            .query();
        expect(query).toEqual(test_queries[105][1]);
    });

    it(`QueryBuilder::search().from(1).where_().not_beyond().ids("a").query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .not_beyond()
            .ids("a")
            .query();
        expect(query).toEqual(test_queries[106][1]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().edge().query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .node()
            .or()
            .edge()
            .query();
        expect(query).toEqual(test_queries[107][1]);
    });

    it(`QueryBuilder::search().from(1).where_().node().and().distance(CountComparison::GreaterThanOrEqual(3)).query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .where()
            .node()
            .and()
            .distance(CountComparison.GreaterThanOrEqual(3))
            .query();
        expect(query).toEqual(test_queries[108][1]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Equal(1.into())).end_where().query()`, () => {
        let query = QueryBuilder.search()
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
        expect(query).toEqual(test_queries[109][1]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(1.into())).end_where().query()`, () => {
        let query = QueryBuilder.search()
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
        expect(query).toEqual(test_queries[110][1]);
    });

    it(`QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(([1,2]).into())).end_where().query()`, () => {
        let query = QueryBuilder.search()
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
        expect(query).toEqual(test_queries[111][1]);
    });

    it(`QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).where_().node().query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .order_by([DbKeyOrder.Asc("k")])
            .where()
            .node()
            .query();
        expect(query).toEqual(test_queries[112][1]);
    });

    it(`QueryBuilder::search().from(1).limit(1).where_().node().query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .limit(1)
            .where()
            .node()
            .query();
        expect(query).toEqual(test_queries[113][1]);
    });

    it(`QueryBuilder::search().from(1).offset(1).where_().node().query()`, () => {
        let query = QueryBuilder.search()
            .from(1)
            .offset(1)
            .where()
            .node()
            .query();
        expect(query).toEqual(test_queries[114][1]);
    });

    it(`QueryBuilder::search().to(1).offset(1).query()`, () => {
        let query = QueryBuilder.search().to(1).offset(1).query();
        expect(query).toEqual(test_queries[115][1]);
    });

    it(`QueryBuilder::search().to(1).limit(1).query()`, () => {
        let query = QueryBuilder.search().to(1).limit(1).query();
        expect(query).toEqual(test_queries[116][1]);
    });

    it(`QueryBuilder::search().to(1).where_().node().query()`, () => {
        let query = QueryBuilder.search().to(1).where().node().query();
        expect(query).toEqual(test_queries[117][1]);
    });

    it(`QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).where_().node().query()`, () => {
        let query = QueryBuilder.search()
            .to(1)
            .order_by([DbKeyOrder.Asc("k")])
            .where()
            .node()
            .query();
        expect(query).toEqual(test_queries[118][1]);
    });
});
