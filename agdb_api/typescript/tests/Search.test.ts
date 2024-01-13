import { QueryBuilder } from "../src/index";
import { describe, it } from "vitest";

describe("search", () => {
    it("search()", () => {
        QueryBuilder.search()
            .depth_first()
            .from(1)
            .to(2)
            .order_by([{ Asc: { String: "key" } }])
            .offset(10)
            .limit(10)
            .where()
            .beyond()
            .edge()
            .and()
            .node()
            .or()
            .not()
            .keys([{ String: "key" }, { String: "key2" }])
            .and()
            .where()
            .distance({ LessThan: 10 })
            .or()
            .edge_count({ GreaterThan: 10 })
            .and()
            .edge_count_from({ Equal: 1 })
            .end_where()
            .or()
            .edge_count_to({ Equal: 1 })
            .and()
            .not_beyond()
            .ids([1, 2])
            .and()
            .key({ String: "key" })
            .value({ LessThan: { U64: 100 } })
            .or()
            .key({ String: "key2" })
            .value({ Contains: { String: "a" } })
            .query();
    });

    it("nested where", () => {
        QueryBuilder.search().from(1).where().node().and().where().edge().query();
    });

    it("search().depth_first().from()", () => {
        QueryBuilder.search().depth_first().from(1).query();
    });

    it("search().depth_first().from().limit().where()", () => {
        QueryBuilder.search().depth_first().from(1).limit(10).where().node().query();
    });

    it("search().depth_first().from().offset().limit()", () => {
        QueryBuilder.search().depth_first().from(1).offset(10).limit(10).query();
    });

    it("search().depth_first().from().offset().limit().where()", () => {
        QueryBuilder.search().depth_first().from(1).offset(10).limit(10).where().node().query();
    });

    it("search().depth_first().from().limit()", () => {
        QueryBuilder.search().depth_first().from(1).limit(10).query();
    });

    it("search().depth_first().from().offset()", () => {
        QueryBuilder.search().depth_first().from(1).offset(10).query();
    });

    it("search().depth_first().from().offset().where()", () => {
        QueryBuilder.search().depth_first().from(1).offset(10).where().node().query();
    });

    it("search().depth_first().from().order_by()", () => {
        QueryBuilder.search()
            .depth_first()
            .from(1)
            .order_by([{ Asc: { String: "key" } }])
            .query();
    });

    it("search().depth_first().from().order_by().where()", () => {
        QueryBuilder.search()
            .depth_first()
            .from(1)
            .order_by([{ Asc: { String: "key" } }])
            .where()
            .node()
            .query();
    });

    it("search().depth_first().from().order_by().limit()", () => {
        QueryBuilder.search()
            .depth_first()
            .from(1)
            .order_by([{ Asc: { String: "key" } }])
            .limit(10)
            .query();
    });

    it("search().depth_first().from().order_by().offset()", () => {
        QueryBuilder.search()
            .depth_first()
            .from(1)
            .order_by([{ Asc: { String: "key" } }])
            .offset(10)
            .query();
    });

    it("search().depth_first().from().where()", () => {
        QueryBuilder.search().depth_first().from(1).where().node().query();
    });

    it("search().depth_first().from().to()", () => {
        QueryBuilder.search().depth_first().from(1).to(1).query();
    });

    it("search().depth_first().to()", () => {
        QueryBuilder.search().depth_first().to(1).query();
    });

    it("search().depth_first().to().limit()", () => {
        QueryBuilder.search().depth_first().to(1).limit(10).query();
    });

    it("search().depth_first().to().offset()", () => {
        QueryBuilder.search().depth_first().to(1).offset(10).query();
    });

    it("search().depth_first().to().order_by()", () => {
        QueryBuilder.search()
            .depth_first()
            .to(1)
            .order_by([{ Asc: { String: "key" } }])
            .query();
    });

    it("search().depth_first().to().where()", () => {
        QueryBuilder.search().depth_first().to(1).where().node().query();
    });

    it("search().breadth_first().from()", () => {
        QueryBuilder.search().breadth_first().from(1).query();
    });

    it("search().breadth_first().to()", () => {
        QueryBuilder.search().breadth_first().to(1).query();
    });

    it("search().from()", () => {
        QueryBuilder.search().from(1).query();
    });

    it("search().index()", () => {
        QueryBuilder.search().index({ String: "key" }).value({ U64: 20 }).query();
    });

    it("search().to()", () => {
        QueryBuilder.search().to(1).query();
    });
});
