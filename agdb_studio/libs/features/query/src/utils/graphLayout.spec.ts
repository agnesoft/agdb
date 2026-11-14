import { describe, expect, it } from "vitest";
import { carryOverLayout } from "./graphLayout";

describe("carryOverLayout", () => {
  it("keeps positions of nodes that stay and reports new ones", () => {
    const previous = [
      { id: 1, x: 1, y: 2, z: 3, vx: 0.5, fx: 9 },
      { id: 2, x: 4 },
    ];
    const next: { id: number }[] = [{ id: 1 }, { id: 3 }];
    expect(carryOverLayout(previous, next)).toBe(true);
    expect(next[0]).toEqual({ id: 1, x: 1, y: 2, z: 3, vx: 0.5, fx: 9 });
    expect(next[1]).toEqual({ id: 3 });
  });

  it("reports no new nodes when nodes are only kept or removed", () => {
    const next: { id: number }[] = [{ id: 1 }];
    expect(carryOverLayout([{ id: 1 }, { id: 2 }], next)).toBe(false);
    // Nodes not laid out yet have nothing to copy.
    expect(next[0]).toEqual({ id: 1 });
  });
});
