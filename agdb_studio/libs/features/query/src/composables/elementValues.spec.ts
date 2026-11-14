import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import type { Components } from "@agnesoft/agdb_api";
import { useElementValues } from "./elementValues";
import type { GraphLink, GraphNode } from "./types";

type DbKeyValue = Components.Schemas.DbKeyValue;

const kv = (key: string, value: number): DbKeyValue => ({
  key: { String: key },
  value: { I64: value },
});
const dbElement = (id: number, values: DbKeyValue[]) => ({
  id,
  from: 0,
  to: 0,
  values,
});
const graphNode = (id: number, rawValues: DbKeyValue[] = []): GraphNode => ({
  id,
  values: [],
  rawValues,
  source: "query",
});

const deferred = <T>() => {
  let resolve: (value: T) => void = () => {};
  let reject: (error: unknown) => void = () => {};
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
};

const exec = vi.fn();

describe("useElementValues", () => {
  beforeEach(() => {
    exec.mockReset();
  });

  it("loads stored values in one request and describes elements with them", async () => {
    exec.mockResolvedValue([
      {
        result: 2,
        elements: [dbElement(1, [kv("age", 30)]), dbElement(-2, [])],
      },
    ]);
    const values = useElementValues(exec);
    await values.load([1, -2, 1, 0]);

    // Duplicates and id 0 (database metadata) are not requested.
    expect(exec).toHaveBeenCalledTimes(1);
    expect(exec).toHaveBeenCalledWith([
      { SelectValues: { keys: [], ids: { Ids: [{ Id: 1 }, { Id: -2 }] } } },
    ]);
    // The query only returned a computed value; the stored one is shown.
    const described = values.describe(graphNode(1, [kv("edge_count", 0)]));
    expect(described.valuesState).toBe("loaded");
    expect(described.rawValues).toEqual([kv("age", 30)]);
    expect(described.values).toEqual([{ key: "age", value: "30" }]);
    expect(described.queryOnlyValues).toEqual([kv("edge_count", 0)]);
  });

  it("does not repeat values the query returned that are stored", async () => {
    exec.mockResolvedValue([
      { result: 1, elements: [dbElement(1, [kv("a", 1), kv("b", 2)])] },
    ]);
    const values = useElementValues(exec);
    await values.load([1]);
    expect(values.describe(graphNode(1, [kv("a", 1)])).queryOnlyValues).toEqual(
      [],
    );
  });

  it("sends large loads as one request per chunk", async () => {
    exec.mockResolvedValue([]);
    const ids = Array.from({ length: 1001 }, (_, i) => i + 1);
    await useElementValues(exec).load(ids);
    expect(exec).toHaveBeenCalledTimes(3);
    expect(exec.mock.calls.map((call) => call[0])).toEqual(
      [500, 500, 1].map((size, i) => [
        {
          SelectValues: {
            keys: [],
            ids: {
              Ids: ids.slice(i * 500, i * 500 + size).map((id) => ({ Id: id })),
            },
          },
        },
      ]),
    );
  });

  it("asks again for the existing ids when the server rejects a chunk", async () => {
    const stored = new Set([1, 2, 4]);
    exec.mockImplementation(async (queries: Components.Schemas.QueryType[]) => {
      const ids =
        queries[0] && "SelectValues" in queries[0]
          ? queries[0].SelectValues.ids
          : undefined;
      if (ids && "Ids" in ids) {
        throw Object.assign(new Error("Id '3' not found"), {
          response: { status: 470 },
        });
      }
      // The search returns only the elements that exist.
      return [
        { result: 3, elements: [...stored].map((id) => dbElement(id, [])) },
      ];
    });
    const values = useElementValues(exec);
    await values.load([1, 2, 3, 4]);
    expect(exec).toHaveBeenCalledTimes(2);
    expect(JSON.stringify(exec.mock.calls[1]?.[0])).toContain('"Search"');
    expect(
      [1, 2, 3, 4].map((id) => values.describe(graphNode(id)).valuesState),
    ).toEqual(["loaded", "loaded", "error", "loaded"]);
  });

  it("marks a rejected chunk failed when the fallback fails too", async () => {
    exec.mockRejectedValue(
      Object.assign(new Error("Id not found"), { response: { status: 470 } }),
    );
    const values = useElementValues(exec);
    await values.load([1, 2]);
    expect(exec).toHaveBeenCalledTimes(2);
    expect(values.describe(graphNode(1)).valuesState).toBe("error");
  });

  it("drops a fallback answer that arrives after the cache was cleared", async () => {
    const late = deferred<unknown>();
    exec
      .mockRejectedValueOnce(
        Object.assign(new Error("Id not found"), { response: { status: 470 } }),
      )
      .mockReturnValueOnce(late.promise);
    const values = useElementValues(exec);
    const loading = values.load([1]);
    await vi.waitFor(() => expect(exec).toHaveBeenCalledTimes(2));
    values.clear();
    late.resolve([{ result: 1, elements: [dbElement(1, [kv("a", 1)])] }]);
    await loading;
    expect(values.describe(graphNode(1)).valuesState).toBeUndefined();
  });

  it("does not ask again when the cache was cleared meanwhile", async () => {
    const late = deferred<unknown>();
    exec.mockReturnValueOnce(late.promise);
    const values = useElementValues(exec);
    const loading = values.load([1]);
    values.clear();
    late.reject(Object.assign(new Error("x"), { response: { status: 470 } }));
    await loading;
    expect(exec).toHaveBeenCalledTimes(1);
  });

  it.each([
    ["a network error", new Error("offline")],
    [
      "a server error",
      Object.assign(new Error("boom"), { response: { status: 500 } }),
    ],
    [
      "an error without a status",
      Object.assign(new Error("x"), { response: null }),
    ],
  ])("does not split on %s", async (_, error) => {
    exec.mockRejectedValue(error);
    const values = useElementValues(exec);
    await values.load([1, 2, 3]);
    expect(exec).toHaveBeenCalledTimes(1);
    expect(values.describe(graphNode(2)).valuesState).toBe("error");
  });

  it("leaves unknown elements unchanged and skips cached ones", async () => {
    exec.mockResolvedValue([{ result: 1, elements: [dbElement(1, [])] }]);
    const values = useElementValues(exec);
    const node = graphNode(5);
    expect(values.describe(node)).toBe(node);
    await values.load([1]);
    await values.load([1]);
    await values.load([]);
    expect(exec).toHaveBeenCalledTimes(1);
  });

  it("reports loading and marks failures, retried only on request", async () => {
    const pending = deferred<unknown>();
    exec.mockReturnValueOnce(pending.promise);
    const values = useElementValues(exec);
    const loading = values.load([1, 2]);
    expect(values.describe(graphNode(1)).valuesState).toBe("loading");
    // Already loading: not requested twice.
    await values.load([1]);
    expect(exec).toHaveBeenCalledTimes(1);

    pending.reject(new Error("offline"));
    await loading;
    const failed = values.describe(graphNode(1, [kv("a", 1)]));
    expect(failed.valuesState).toBe("error");
    expect(failed.rawValues).toEqual([kv("a", 1)]);

    await values.load([1]);
    expect(exec).toHaveBeenCalledTimes(1);
    exec.mockResolvedValueOnce([{ result: 1, elements: [dbElement(1, [])] }]);
    await values.load([1, 2], { retry: true });
    expect(exec).toHaveBeenLastCalledWith([
      { SelectValues: { keys: [], ids: { Ids: [{ Id: 1 }, { Id: 2 }] } } },
    ]);
    expect(values.describe(graphNode(1)).valuesState).toBe("loaded");
    // Not returned by the database (e.g. deleted elsewhere).
    expect(values.describe(graphNode(2)).valuesState).toBe("error");
  });

  it("marks everything failed when there is no database to ask", async () => {
    exec.mockResolvedValue(undefined);
    const values = useElementValues(exec);
    await values.load([1]);
    expect(values.describe(graphNode(1)).valuesState).toBe("error");
  });

  it("drops responses that arrive after the cache was cleared", async () => {
    const late = deferred<unknown>();
    exec.mockReturnValueOnce(late.promise);
    const values = useElementValues(exec);
    const loading = values.load([1]);
    values.clear();
    late.resolve([{ result: 1, elements: [dbElement(1, [kv("old", 1)])] }]);
    await loading;
    expect(values.describe(graphNode(1)).valuesState).toBeUndefined();

    const failing = deferred<unknown>();
    exec.mockReturnValueOnce(failing.promise);
    const failingLoad = values.load([1]);
    values.clear();
    failing.reject(new Error("late"));
    await failingLoad;
    expect(values.describe(graphNode(1)).valuesState).toBeUndefined();
  });

  it("sets and removes values directly", () => {
    const values = useElementValues(exec);
    const link: GraphLink = {
      id: -3,
      values: [],
      rawValues: [],
      source: 1,
      target: 2,
      elementSource: "query",
    };
    values.set(-3, [kv("w", 2)]);
    expect(values.describe(link).rawValues).toEqual([kv("w", 2)]);
    expect(values.describe(link).source).toBe(1);
    values.remove(-3);
    expect(values.describe(link)).toBe(link);
  });
});
