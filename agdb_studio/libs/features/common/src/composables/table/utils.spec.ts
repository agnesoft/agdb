import { dateFormatter } from "./utils";
import { describe, it, expect } from "vitest";

describe("utils", () => {
  describe("dateFormatter", () => {
    it("should return a formatted date when value is number", () => {
      const formattedDate = dateFormatter(1734447876);
      expect(formattedDate).toBe(new Date(1734447876000).toUTCString());
    });
    it("should return N/A when value is not a number", () => {
      const formattedDate = dateFormatter("not a number");
      expect(formattedDate).toBe("N/A");
    });
    it("should return N/A when value is 0", () => {
      const formattedDate = dateFormatter(0);
      expect(formattedDate).toBe("N/A");
    });
  });
});
