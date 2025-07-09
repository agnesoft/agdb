import { TABLE_NAME, tableConfig, tableData } from "../../tests/tableMocks";
import { addTable } from "./tableConfig";
import { setTableData } from "./tableData";
import {
  addFilter,
  getTableFilter,
  removeFilter,
  setSort,
} from "./tableFilter";
import { describe, beforeEach, it, expect, vi } from "vitest";

describe("tableFilter", () => {
  addTable({ name: TABLE_NAME, columns: tableConfig, fetchData: vi.fn() });
  setTableData(TABLE_NAME, tableData);

  beforeEach(() => {
    const filter = getTableFilter(TABLE_NAME);
    filter.filters.clear();
    filter.sort.clear();
  });

  describe("getTableFilter", () => {
    it("should return empty table filter", () => {
      const filter = getTableFilter("non_existent_table");
      expect(filter).toEqual({
        filters: new Map(),
        sort: new Map(),
      });
    });
    it("should return table filter", () => {
      const filter = getTableFilter(TABLE_NAME);
      expect(filter).toEqual({
        filters: new Map(),
        sort: new Map(),
      });
    });
  });

  describe("addFilter", () => {
    it("should add filter", () => {
      addFilter(TABLE_NAME, "role", "admin");
      const filter = getTableFilter(TABLE_NAME);
      expect(filter.filters.size).toBe(1);
      expect(filter.filters.get("role")).toBe("admin");
    });
  });

  describe("removeFilter", () => {
    it("should remove filter", () => {
      const filter = getTableFilter(TABLE_NAME);
      addFilter(TABLE_NAME, "role", "admin");
      expect(filter.filters.size).toBe(1);
      removeFilter(TABLE_NAME, "role");
      expect(filter.filters.size).toBe(0);
    });
  });

  describe("addSort", () => {
    it("should add sort", () => {
      setSort(TABLE_NAME, "name", "asc");
      const filter = getTableFilter(TABLE_NAME);
      expect(filter.sort.size).toBe(1);
      expect(filter.sort.get("name")).toBe("asc");
    });
  });
});
