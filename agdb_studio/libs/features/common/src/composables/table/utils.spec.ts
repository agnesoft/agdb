import { dateFormatter, numberFormatter } from "./utils";
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
  describe("numberFormatter", () => {
    it("should return a formatted number when value is number", () => {
      const formattedNumber = numberFormatter(1234567.12);
      expect(formattedNumber).toBe("1,234,567.12");
    });
    it("should return N/A when value is not a number", () => {
      const formattedNumber = numberFormatter("not a number");
      expect(formattedNumber).toBe("N/A");
    });
    it("should return N/A when value is 0", () => {
      const formattedNumber = numberFormatter(0);
      expect(formattedNumber).toBe("N/A");
    });
  });
});
