import { QueryBuilder } from "../src/query_builder";
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
});
