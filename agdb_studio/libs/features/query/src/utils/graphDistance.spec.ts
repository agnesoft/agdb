import { describe, expect, it } from "vitest";
import { distanceFrom, searchStart } from "./graphDistance";

const link = (source: unknown, target: unknown) => ({ source, target });

describe("distanceFrom", () => {
  it("counts hops either way along the edges, leaving out what it doesn't reach", () => {
    // 1 -> 2 -> 3, 4 -> 3 (into it), 5 alone; 2 as a node object, as the graph makes it.
    const hops = distanceFrom(1, [
      link(1, { id: 2 }),
      link(2, 3),
      link(4, 3),
      link(1, 3),
    ]);
    expect(Object.fromEntries(hops)).toEqual({ 1: 0, 2: 1, 3: 1, 4: 2 });
    expect(hops.has(5)).toBe(false);
  });

  it("knows only the start without edges", () => {
    expect([...distanceFrom(7, [])]).toEqual([[7, 0]]);
  });
});

describe("searchStart", () => {
  const nodes = (alias: string) => (alias === "root" ? 2 : undefined);
  const search = (origin: unknown, destination: unknown = { Id: 0 }) => ({
    Search: { algorithm: "BreadthFirst", origin, destination, limit: 0 },
  });

  it("is the origin of the first search, by id or alias", () => {
    expect(searchStart([search({ Id: 5 })], nodes)).toBe(5);
    expect(searchStart([search({ Alias: "root" })], nodes)).toBe(2);
    expect(searchStart([search({ Alias: "gone" })], nodes)).toBeUndefined();
  });

  it("is the destination of a search backwards", () => {
    expect(searchStart([search({ Id: 0 }, { Id: 3 })], nodes)).toBe(3);
  });

  it("finds a search nested in a select, after queries without one", () => {
    const select = {
      SelectValues: { keys: [], ids: { Search: search({ Id: 4 }).Search } },
    };
    expect(searchStart([{ InsertNodes: { count: 1 } }, select], nodes)).toBe(4);
  });

  it("is none without a search or a start", () => {
    expect(searchStart([], nodes)).toBeUndefined();
    expect(
      searchStart([{ SelectNodeCount: {} }, "text"], nodes),
    ).toBeUndefined();
    expect(searchStart([search({ Id: 0 })], nodes)).toBeUndefined();
    // A query seen twice is walked once.
    const shared = { a: 1 };
    expect(searchStart([{ x: shared, y: shared }], nodes)).toBeUndefined();
  });
});
