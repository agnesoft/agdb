import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  copySteps,
  formatSteps,
  parseSteps,
  pastedQuery,
  readCopiedSteps,
  resetCopiedSteps,
} from "./queryRowClipboard";
import type { QueryStep } from "../composables/types";

const ids = [[{ selectedOption: "signed", value: "1" }]];
const steps: QueryStep[] = [
  { id: "s1", type: "select", invalid: false },
  { id: "s2", type: "select.ids", args: ids, invalid: true },
];
const copied = [{ type: "select" }, { type: "select.ids", args: ids }];
const CODE = "QueryBuilder::select().ids(1).query()";

const stubClipboard = (clipboard?: Partial<Clipboard>) =>
  vi.stubGlobal("navigator", { clipboard });

describe("queryRowClipboard", () => {
  beforeEach(() => resetCopiedSteps());
  afterEach(() => vi.unstubAllGlobals());

  it("copies as Rust code and pastes it back without ids", () => {
    expect(formatSteps(steps)).toBe(CODE);
    expect(parseSteps(CODE)).toEqual({ steps: copied });
  });

  it("explains why text is not a query", () => {
    expect(parseSteps("  ")).toEqual({
      error: "The clipboard does not contain a query",
    });
    expect(parseSteps("select().nope()")).toEqual({
      error: "nope() cannot follow select()",
    });
    expect(parseSteps("select(")).toEqual({
      error: "Unexpected end of the query",
    });
  });

  it("only treats method chains as pasted queries", () => {
    expect(pastedQuery("select")).toBeUndefined();
    expect(pastedQuery("some.text")).toBeUndefined();
    expect(pastedQuery("select.node_count")).toEqual([
      { type: "select" },
      { type: "select.node_count" },
    ]);
  });

  it("copies to and reads from the system clipboard", async () => {
    let stored = "";
    stubClipboard({
      writeText: vi.fn(async (text: string) => {
        stored = text;
      }),
      readText: vi.fn(async () => stored),
    });
    expect(await copySteps(steps)).toBe("copied");
    expect(stored).toBe(CODE);
    expect(await readCopiedSteps()).toEqual({ steps: copied });
  });

  it("reports other clipboard text", async () => {
    stubClipboard({ readText: vi.fn().mockResolvedValue("some text") });
    expect(await readCopiedSteps()).toHaveProperty("error");
  });

  it("reports denied clipboard access without falling back", async () => {
    stubClipboard({
      writeText: vi.fn().mockRejectedValue(new Error("denied")),
      readText: vi.fn().mockRejectedValue(new Error("denied")),
    });
    expect(await copySteps(steps)).toBe("failed");
    expect(await readCopiedSteps()).toEqual({ denied: true });
  });

  it("keeps a local copy when the browser has no clipboard API", async () => {
    stubClipboard(undefined);
    expect(await readCopiedSteps()).toHaveProperty("error");
    expect(await copySteps(steps)).toBe("local");
    expect(await readCopiedSteps()).toEqual({ steps: copied });
  });
});
