import { describe, expect, it } from "vitest";
import { stepMark } from "./chartKeys";

describe("stepMark", () => {
  it("steps through the marks and jumps to either end", () => {
    expect(stepMark("ArrowRight", undefined, 3)).toBe(0);
    expect(stepMark("ArrowDown", 1, 3)).toBe(2);
    expect(stepMark("ArrowRight", 2, 3)).toBe(2);
    expect(stepMark("ArrowLeft", undefined, 3)).toBe(2);
    expect(stepMark("ArrowUp", 0, 3)).toBe(0);
    expect(stepMark("Home", 2, 3)).toBe(0);
    expect(stepMark("End", 0, 3)).toBe(2);
  });

  it("ignores other keys and empty charts", () => {
    expect(stepMark("Enter", 0, 3)).toBeUndefined();
    expect(stepMark("ArrowRight", undefined, 0)).toBeUndefined();
  });
});
