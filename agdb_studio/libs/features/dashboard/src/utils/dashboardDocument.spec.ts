import { describe, expect, it } from "vitest";
import {
  addToDocument,
  createWidget,
  emptyDocument,
  moveInDocument,
  NewerDashboardError,
  orderedWidgets,
  parseDocument,
  removeFromDocument,
  resizeInDocument,
  serializeDocument,
  validateDashboardName,
} from "./dashboardDocument";
import type { DashboardDocument } from "../types";

const widget = (id: string) => ({
  ...createWidget({ owner: "o", db: "d" }),
  id,
});

const ids = (doc: DashboardDocument) =>
  orderedWidgets(doc).map(({ widget }) => widget.id);

describe("dashboardDocument", () => {
  it("creates widgets with defaults and a unique id", () => {
    const a = createWidget({ owner: "o", db: "d" });
    const b = createWidget({ owner: "o", db: "d", view: "scalar" });
    expect(a).toMatchObject({
      title: "New widget",
      rows: { exec: [], context: [] },
    });
    expect(a.view).toBeUndefined();
    expect(b.view).toBe("scalar");
    expect(a.id).not.toBe(b.id);
  });

  it("round-trips a document with rows, colours, view and layout", () => {
    const doc = emptyDocument();
    addToDocument(doc, [
      createWidget({
        owner: "o",
        db: "d",
        rows: {
          exec: [{ id: "r1", query: 'select().ids("users")' }],
          context: [{ id: "r2", query: "select().ids(1)" }],
        },
        view: "graph",
        colors: {
          rules: [
            {
              kind: "property",
              id: "p",
              key: "k",
              op: "eq",
              value: "v",
              color: 1,
            },
            { kind: "property", id: "q", key: "k", op: "exists", color: 2 },
            { kind: "palette", id: "c", key: "type" },
          ],
          rows: { r1: 3 },
          elements: { 5: 4 },
        },
        refreshSeconds: 30,
      }),
    ]);
    addToDocument(doc, [widget("b")], { cols: 4, rows: 3 });
    expect(parseDocument(serializeDocument(doc))).toEqual(doc);
  });

  it("refuses a document from a newer Studio", () => {
    expect(() => parseDocument('{"version":3,"widgets":{}}')).toThrow(
      NewerDashboardError,
    );
    expect(parseDocument('{"version":2,"widgets":{}}')).toEqual(
      emptyDocument(),
    );
  });

  it("gives an empty document for broken configs", () => {
    expect(parseDocument("not json")).toEqual(emptyDocument());
    expect(parseDocument("[]")).toEqual(emptyDocument());
    expect(parseDocument('{"widgets":"x"}')).toEqual(emptyDocument());
    expect(parseDocument('{"widgets":{}}')).toEqual(emptyDocument());
  });

  it("drops broken widgets, rows and colours and fills defaults", () => {
    const doc = parseDocument(
      JSON.stringify({
        widgets: {
          n: null,
          x: { owner: "o" },
          w: {
            id: "ignored: the key is the id",
            owner: "o",
            db: "d",
            rows: {
              exec: [{ id: "r", query: "q" }, { id: 1 }, null],
              context: "x",
            },
            view: "bogus",
            colors: {
              rules: [
                null,
                { id: "a" },
                { kind: "property", id: "b", key: "k", color: -1 },
                { kind: "other", id: "c", key: "k" },
                { kind: "property", id: "d", key: "k", op: "nope", color: 0 },
                { kind: "property", id: "e", key: "k", color: 1, value: 5 },
              ],
              rows: { r: 1, s: "red", t: 1.5 },
              elements: { 3: 2, x: 1 },
            },
            refreshSeconds: 3,
          },
          v: {
            title: "T",
            owner: "o",
            db: "d",
            colors: { rules: "x", rows: 5 },
            refreshSeconds: 86_401,
          },
        },
        layout: [{ id: "v", cols: 4, rows: 3 }],
      }),
    );
    expect(doc.widgets).toEqual({
      w: {
        id: "w",
        title: "",
        owner: "o",
        db: "d",
        rows: { exec: [{ id: "r", query: "q" }], context: [] },
        colors: {
          rules: [
            { kind: "property", id: "d", key: "k", op: "eq", color: 0 },
            { kind: "property", id: "e", key: "k", op: "eq", color: 1 },
          ],
          rows: { r: 1 },
          elements: { 3: 2 },
        },
      },
      v: {
        id: "v",
        title: "T",
        owner: "o",
        db: "d",
        rows: { exec: [], context: [] },
        colors: { rules: [], rows: {}, elements: {} },
      },
    });
    // The layout's widgets first; one it misses goes last at the default size.
    expect(doc.layout).toEqual([
      { id: "v", cols: 4, rows: 3 },
      { id: "w", cols: 2, rows: 1 },
    ]);
  });

  it("drops broken, unknown and repeated layout entries", () => {
    const doc = parseDocument(
      JSON.stringify({
        widgets: { a: widget("a"), b: widget("b") },
        layout: [
          null,
          { id: 1 },
          { id: "gone" },
          { id: "constructor" },
          { id: "b", cols: 9, rows: "2" },
          { id: "a", cols: 1, rows: 2 },
          { id: "b", cols: 4, rows: 3 },
        ],
      }),
    );
    expect(doc.layout).toEqual([
      { id: "b", cols: 2, rows: 1 },
      { id: "a", cols: 1, rows: 2 },
    ]);
  });

  it("does not let a widget id reach the object prototype", () => {
    const doc = parseDocument(
      '{"widgets":{"__proto__":{"owner":"o","db":"d"}},"layout":[{"id":"__proto__"}]}',
    );
    expect(doc).toEqual(emptyDocument());
    expect(Object.getPrototypeOf(doc.widgets)).toBe(Object.prototype);
  });

  it("reads older configs that listed widgets with their sizes", () => {
    const doc = parseDocument(
      JSON.stringify({
        version: 2,
        widgets: [
          { ...widget("a"), size: { cols: 3, rows: 2 } },
          { ...widget("b"), size: "big" },
          { ...widget("a"), title: "repeated" },
          null,
          { ...widget("__proto__") },
        ],
      }),
    );
    expect(Object.keys(doc.widgets)).toEqual(["a", "b"]);
    expect(doc.widgets.a).not.toHaveProperty("size");
    expect(doc.widgets.a!.title).toBe("New widget");
    expect(doc.layout).toEqual([
      { id: "a", cols: 3, rows: 2 },
      { id: "b", cols: 2, rows: 1 },
    ]);
  });

  it("migrates version 1 widgets (one query, fixed view, charts as tables)", () => {
    const v1 = (view: string) => ({
      id: view,
      title: view,
      owner: "o",
      db: "d",
      query: "select().ids(1)",
      view,
      size: { cols: 1, rows: 1 },
    });
    const doc = parseDocument(
      JSON.stringify({
        version: 1,
        widgets: [v1("stat"), v1("chart"), v1("graph"), v1("table")],
      }),
    );
    expect(doc.version).toBe(2);
    expect(orderedWidgets(doc).map(({ widget }) => widget.view)).toEqual([
      "scalar",
      "table",
      "graph",
      "table",
    ]);
    expect(doc.widgets.stat!.rows.exec).toEqual([
      { id: "stat-row", query: "select().ids(1)" },
    ]);
    expect(doc.layout[0]).toEqual({ id: "stat", cols: 1, rows: 1 });
  });

  it("adds, replaces, moves, resizes and removes widgets", () => {
    const doc = emptyDocument();
    addToDocument(doc, [widget("a"), widget("b"), widget("c")]);
    // Adding an existing widget replaces its content and keeps its place.
    addToDocument(doc, [{ ...widget("a"), title: "Edited" }], {
      cols: 4,
      rows: 3,
    });
    expect(ids(doc)).toEqual(["a", "b", "c"]);
    expect(doc.widgets.a!.title).toBe("Edited");
    expect(doc.layout[0]).toEqual({ id: "a", cols: 2, rows: 1 });

    expect(moveInDocument(doc, "a", 1)).toBe(true);
    expect(moveInDocument(doc, "a", 5)).toBe(false);
    expect(moveInDocument(doc, "b", -1)).toBe(false);
    expect(moveInDocument(doc, "missing", 1)).toBe(false);
    expect(ids(doc)).toEqual(["b", "a", "c"]);

    expect(resizeInDocument(doc, "c", { cols: 4, rows: 2 })).toBe(true);
    expect(resizeInDocument(doc, "missing", { cols: 1, rows: 1 })).toBe(false);
    expect(orderedWidgets(doc)[2]!.layout).toEqual({
      id: "c",
      cols: 4,
      rows: 2,
    });

    removeFromDocument(doc, "a");
    expect(ids(doc)).toEqual(["b", "c"]);
    expect(Object.keys(doc.widgets)).toEqual(["b", "c"]);
  });

  it("skips layout entries without a widget when ordering", () => {
    const doc = emptyDocument();
    addToDocument(doc, [widget("a")]);
    doc.layout.push({ id: "gone", cols: 1, rows: 1 });
    expect(ids(doc)).toEqual(["a"]);
  });

  it("validates dashboard names like the server", () => {
    expect(validateDashboardName("ops-2026_q1.v2")).toBeUndefined();
    expect(validateDashboardName("list")).toContain("reserved");
    expect(validateDashboardName("")).toBeDefined();
    expect(validateDashboardName("has space")).toBeDefined();
    expect(validateDashboardName("x".repeat(65))).toBeDefined();
    expect(validateDashboardName(".")).toBeDefined();
    expect(validateDashboardName("..")).toBeDefined();
    expect(validateDashboardName("x".repeat(64))).toBeUndefined();
  });
});
