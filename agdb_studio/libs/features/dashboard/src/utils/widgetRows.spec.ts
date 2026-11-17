import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});

import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import { emptyColorConfig } from "@agdb-studio/query/src/utils/graphColorRules";
import { createWidget } from "./dashboardDocument";
import {
  fromQueryRows,
  hasUnfinishedRow,
  seedQuery,
  toQueryRows,
  widgetFromQuery,
} from "./widgetRows";

const USERS = 'select().ids("users")';

describe("widgetRows", () => {
  const queryStore = useQueryStore();

  beforeEach(() => {
    queryStore.clearQueries();
  });

  it("finds rows that cannot run yet", () => {
    const query = queryStore.addQuery({ id: "q" }).value;
    query.rows.exec = toQueryRows([{ id: "r1", query: USERS }]);
    // Empty rows are ignored.
    query.rows.context = [{ id: "c", steps: [] }];
    expect(hasUnfinishedRow(query)).toBe(false);
    query.rows.context = toQueryRows([{ id: "c", query: "select()" }]);
    expect(hasUnfinishedRow(query)).toBe(true);
  });

  it("turns widget rows into builder rows and back, keeping row ids", () => {
    const rows = toQueryRows([
      { id: "r1", query: USERS },
      { id: "bad", query: "not a query" },
      { id: "r2", query: "select().ids(1)" },
    ]);
    expect(rows.map((r) => r.id)).toEqual(["r1", "r2"]);
    expect(rows[0]!.steps.map((s) => s.type)).toEqual(["select", "select.ids"]);
    const stepIds = rows.flatMap((r) => r.steps.map((s) => s.id));
    expect(new Set(stepIds).size).toBe(stepIds.length);
    expect(fromQueryRows(rows)).toEqual([
      { id: "r1", query: USERS },
      { id: "r2", query: "select().ids(1)" },
    ]);
  });

  it("gives an empty row to type into when there are none", () => {
    const rows = toQueryRows([]);
    expect(rows).toHaveLength(1);
    expect(rows[0]!.steps).toEqual([]);
    expect(fromQueryRows(rows)).toEqual([]);
  });

  it("seeds a query entry with rows, colours and view, and reads it back", () => {
    const colors = { ...emptyColorConfig(), rows: { r1: 2 } };
    const widget = createWidget({
      owner: "o",
      db: "d",
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
      colors,
      view: "table",
    });
    seedQuery("q", widget);
    const query = queryStore.getQuery("q")!.value;
    expect(query.rows.exec[0]!.id).toBe("r1");
    expect(query.rows.context).toHaveLength(1);
    expect(query.view).toBe("table");
    expect(query.colors).toEqual(colors);
    expect(query.colors).not.toBe(colors);

    // Seeding again reuses the entry; no colours or view clears them.
    seedQuery("q", { ...widget, colors: undefined, view: undefined });
    expect(queryStore.getQuery("q")!.value.colors).toBeUndefined();
    expect(queryStore.getQuery("q")!.value.view).toBeUndefined();
  });

  it("builds the widget from an edited entry", () => {
    const base = createWidget({
      owner: "o",
      db: "d",
      view: "json",
      colors: { ...emptyColorConfig(), elements: { 1: 1 } },
    });
    seedQuery("q", createWidget({ owner: "o", db: "d" }));
    const query = queryStore.getQuery("q")!;
    query.value.rows.exec = toQueryRows([{ id: "r1", query: USERS }]);
    query.value.rows.context = toQueryRows([
      { id: "r2", query: "select().ids(1)" },
    ]);

    // No view picked and no colouring: the widget's old ones are dropped.
    query.value.colors = emptyColorConfig();
    const plain = widgetFromQuery(base, query.value);
    expect(plain.rows).toEqual({
      exec: [{ id: "r1", query: USERS }],
      context: [{ id: "r2", query: "select().ids(1)" }],
    });
    expect(plain.view).toBeUndefined();
    expect(plain.colors).toBeUndefined();
    expect(plain.id).toBe(base.id);

    query.value.view = "graph";
    query.value.colors = { ...emptyColorConfig(), rows: { r1: 0 } };
    const styled = widgetFromQuery(base, query.value);
    expect(styled.view).toBe("graph");
    expect(styled.colors?.rows).toEqual({ r1: 0 });
  });
});
