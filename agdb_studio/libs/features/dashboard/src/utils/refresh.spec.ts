import { describe, expect, it } from "vitest";
import {
  MAX_REFRESH_SECONDS,
  MIN_REFRESH_SECONDS,
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
