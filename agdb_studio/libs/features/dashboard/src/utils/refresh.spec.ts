import { describe, expect, it } from "vitest";
import {
  MAX_REFRESH_SECONDS,
  MIN_REFRESH_SECONDS,
  REFRESH_CHOICES,
  formatRefresh,
  isRefreshInterval,
} from "./refresh";

describe("isRefreshInterval", () => {
  it.each([
    [MIN_REFRESH_SECONDS, true],
    [60, true],
    [MAX_REFRESH_SECONDS, true],
    [0, false],
    [-10, false],
    [MIN_REFRESH_SECONDS - 1, false],
    [MAX_REFRESH_SECONDS + 1, false],
    [Number.POSITIVE_INFINITY, false],
    [Number.NaN, false],
  ])("%s seconds -> %s", (seconds, expected) => {
    expect(isRefreshInterval(seconds)).toBe(expected);
  });
});

describe("formatRefresh", () => {
  it.each([
    [0, "Off"],
    [10, "10 s"],
    [300, "5 min"],
    [3600, "1 h"],
    [5430, "1 h 30 min 30 s"],
  ])("%s seconds -> %s", (seconds, expected) => {
    expect(formatRefresh(seconds)).toBe(expected);
  });

  it("offers only valid intervals besides off", () => {
    expect(REFRESH_CHOICES.filter((s) => s && !isRefreshInterval(s))).toEqual(
      [],
    );
  });
});
