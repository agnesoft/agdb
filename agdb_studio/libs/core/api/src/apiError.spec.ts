import { AgdbApiError } from "@agnesoft/agdb_api";
import { describe, expect, it } from "vitest";
import { errorMessage } from "./apiError";

describe("errorMessage", () => {
  it.each([
    [new AgdbApiError(470, "Id '3' not found"), "Id '3' not found"],
    [new AgdbApiError(403, ""), "403: "],
    [new Error("network"), "network"],
    ["plain", "plain"],
  ])("explains errors (%#)", (error, message) => {
    expect(errorMessage(error)).toBe(message);
  });
});
