import { describe, expect, it } from "vitest";
import {
  OTHER_COLOR,
  barPath,
  formatCompact,
  formatValue,
  maxOf,
  minOf,
  paletteColor,
  truncate,
  valueScale,
} from "./chartScale";

describe("chartScale", () => {
  it("finds the smallest and largest of many values", () => {
    // More values than a spread call takes as arguments.
    const many = Array.from({ length: 200_000 }, (_, i) => i - 7);
    expect(minOf(many)).toBe(-7);
    expect(maxOf(many)).toBe(199_992);
    expect(minOf([])).toBe(Infinity);
    expect(maxOf([])).toBe(-Infinity);
  });

  it("makes round scales that include zero", () => {
    expect(valueScale([3, 17])).toEqual({
      min: 0,
      max: 20,
      ticks: [0, 5, 10, 15, 20],
    });
    expect(valueScale([-12, 4]).min).toBe(-15);
    expect(valueScale([0.12, 0.34]).ticks).toEqual([0, 0.1, 0.2, 0.3, 0.4]);
    // All zero still gives an axis.
    expect(valueScale([0])).toMatchObject({ min: 0, max: 1 });
    // Values too large for a span of 1 still get a scale (and terminate).
    const huge = valueScale([1e17], 4, false);
    expect(huge.max).toBeGreaterThan(huge.min);
    expect(huge.ticks.length).toBeGreaterThan(1);
    expect(huge.ticks.length).toBeLessThan(10);
    // Whole numbers keep whole ticks.
    expect(valueScale([1, 1]).ticks).toEqual([0, 1]);
    expect(valueScale([2, 3]).ticks).toEqual([0, 1, 2, 3]);
    expect(valueScale([0.5, 1]).ticks).toContain(0.2);
    expect(valueScale([700, 4100]).ticks).toEqual([
      0, 1000, 2000, 3000, 4000, 5000,
    ]);
  });

  it("leaves zero out when asked to", () => {
    expect(valueScale([100, 120], 4, false)).toEqual({
      min: 100,
      max: 120,
      ticks: [100, 105, 110, 115, 120],
    });
    // A flat line in the middle.
    expect(valueScale([50], 4, false)).toMatchObject({ min: 49, max: 51 });
    expect(valueScale([1, 1], 4, false).ticks).toEqual([0, 1, 2]);
  });

  it("formats values and labels", () => {
    expect(formatCompact(1234)).toBe("1.2K");
    expect(formatValue(1234.5678)).toBe("1,234.57");
    expect(truncate("short", 10)).toBe("short");
    expect(truncate("a long label", 6)).toBe("a lon…");
    expect(truncate("abc", 0)).toBe("a…");
  });

  it("names theme colours", () => {
    expect(paletteColor(0)).toBe("var(--graph-palette-1)");
    expect(OTHER_COLOR).toBe("var(--graph-uncolored-color)");
  });

  it("rounds only the data end of bars, in every direction", () => {
    // A column growing up from y 100 to 40.
    expect(barPath(true, 10, 20, 100, 40)).toBe(
      "M10,100 L10,44 A4,4 0 0 1 14,40 L26,40 A4,4 0 0 1 30,44 L30,100 Z",
    );
    // A column growing down (a negative value).
    expect(barPath(true, 10, 20, 100, 120)).toBe(
      "M10,100 L10,116 A4,4 0 0 0 14,120 L26,120 A4,4 0 0 0 30,116 L30,100 Z",
    );
    // A bar growing right, and one growing left.
    expect(barPath(false, 0, 10, 50, 90)).toContain("A4,4 0 0 1 90,4");
    expect(barPath(false, 0, 10, 50, 10)).toContain("A4,4 0 0 0 10,4");
    // Short and thin bars get a smaller radius.
    expect(barPath(true, 0, 4, 10, 8)).toContain("A2,2");
  });
});
