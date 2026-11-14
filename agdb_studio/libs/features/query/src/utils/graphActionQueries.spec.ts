import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import {
  deleteElementQuery,
  insertEdgeQuery,
  neighboursQueries,
  selectElementQuery,
  selectExistingElementsQuery,
  updateValuesQueries,
} from "./graphActionQueries";

const kv = (key: string, value: string) => ({
  key: { String: key },
  value: { String: value },
});

describe("graphActionQueries", () => {
  it("builds a select by id query", () => {
    expect(selectElementQuery(5)).toEqual({
      SelectValues: { keys: [], ids: { Ids: [{ Id: 5 }] } },
    });
    expect(selectElementQuery([5, -6])).toEqual({
      SelectValues: { keys: [], ids: { Ids: [{ Id: 5 }, { Id: -6 }] } },
    });
  });

  it("builds a select of the existing elements among ids", () => {
    expect(selectExistingElementsQuery([5, -6])).toEqual({
      SelectValues: {
        keys: [],
        ids: {
          Search: {
            algorithm: "Elements",
            conditions: [
              {
                data: { Ids: [{ Id: 5 }, { Id: -6 }] },
                logic: "And",
                modifier: "None",
              },
            ],
            origin: { Id: 0 },
            destination: { Id: 0 },
            limit: 0,
            offset: 0,
            order_by: [],
            reverse: false,
          },
        },
      },
    });
  });

  it("builds a remove by id query", () => {
    expect(deleteElementQuery(-3)).toEqual({ Remove: { Ids: [{ Id: -3 }] } });
  });

  it("removes deleted keys and inserts edited values", () => {
    const queries = updateValuesQueries(
      1,
      [kv("a", "1"), kv("b", "2")],
      [kv("a", "10"), kv("c", "3")],
    );
    expect(queries).toEqual([
      {
        RemoveValues: {
          ids: { Ids: [{ Id: 1 }] },
          keys: [{ String: "b" }],
        },
      },
      {
        InsertValues: {
          ids: { Ids: [{ Id: 1 }] },
          values: { Multi: [[kv("a", "10"), kv("c", "3")]] },
        },
      },
    ]);
  });

  it("writes only changed values and nothing when unchanged", () => {
    const queries = updateValuesQueries(
      1,
      [kv("a", "1"), kv("b", "2")],
      [kv("a", "1"), kv("b", "3")],
    );
    expect(queries).toEqual([
      {
        InsertValues: {
          ids: { Ids: [{ Id: 1 }] },
          values: { Multi: [[kv("b", "3")]] },
        },
      },
    ]);
    expect(updateValuesQueries(1, [kv("a", "1")], [kv("a", "1")])).toEqual([]);
  });

  it("omits the remove query when no keys were removed", () => {
    const queries = updateValuesQueries(1, [kv("a", "1")], [kv("a", "2")]);
    expect(queries).toHaveLength(1);
    expect(queries[0]).toHaveProperty("InsertValues");
  });

  it("omits the insert query when all values were removed", () => {
    const queries = updateValuesQueries(1, [kv("a", "1")], []);
    expect(queries).toHaveLength(1);
    expect(queries[0]).toHaveProperty("RemoveValues");
  });

  it("builds an insert edge query without values", () => {
    expect(insertEdgeQuery(1, 2)).toMatchObject({
      InsertEdges: {
        from: { Ids: [{ Id: 1 }] },
        to: { Ids: [{ Id: 2 }] },
        values: { Single: [] },
      },
    });
  });

  it("builds an insert edge query with values", () => {
    expect(insertEdgeQuery(1, 2, [kv("w", "x")])).toMatchObject({
      InsertEdges: { values: { Single: [kv("w", "x")] } },
    });
  });

  it("builds neighbour searches in both directions", () => {
    const [outgoing, incoming] = neighboursQueries(7);
    expect(outgoing).toMatchObject({
      SelectValues: {
        ids: {
          Search: {
            origin: { Id: 7 },
            conditions: [{ data: { Distance: { LessThanOrEqual: 2 } } }],
          },
        },
      },
    });
    expect(incoming).toMatchObject({
      SelectValues: {
        ids: { Search: { destination: { Id: 7 } } },
      },
    });
  });
});
