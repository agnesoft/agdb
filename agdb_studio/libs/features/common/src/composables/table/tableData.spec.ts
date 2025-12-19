import { addTable, getTable } from "./tableConfig";
import {
  addRow,
  removeRow,
  setTableData,
  clearTableData,
  getRows,
} from "./tableData";
import { TABLE_NAME, tableConfig, tableData } from "../../tests/tableMocks";
import { addFilter, getTableFilter, setSort } from "./tableFilter";
import { describe, beforeEach, it, expect, vi } from "vitest";
import type { TRow } from "./types";

describe("tableData", () => {
  const fetchDataMock = vi.fn();
  addTable({
    name: TABLE_NAME,
    columns: tableConfig,
    uniqueKey: (row: TRow) => `${row.owner?.toString()}/${row.db?.toString()}`,
    fetchData: fetchDataMock,
  });

  beforeEach(() => {
    const table = getTable(TABLE_NAME);
    table?.data?.clear();
  });

  describe("setTableData", () => {
    it("should set table data", () => {
      const table = getTable(TABLE_NAME);
      expect(table?.data?.size).toBe(0);
      setTableData(TABLE_NAME, tableData);
      expect(table?.data?.size).toBe(5);
    });
    it("should not set table data if table does not exist", () => {
      const table = getTable("non_existent_table");
      expect(table).toBeUndefined();
      setTableData("non_existent_table", tableData);
      expect(table).toBeUndefined();
    });
    it("should set table data with default keys if unique key doesn't exist", () => {
      addTable({
        name: "table_without_unique_key",
        columns: tableConfig,
        fetchData: fetchDataMock,
      });
      setTableData("table_without_unique_key", tableData);
      const table = getTable("table_without_unique_key");
      expect(table?.data?.size).toBe(5);
    });
    it("should set table data with custom string unique key", () => {
      addTable({
        name: "table_with_string_unique_key",
        columns: [
          { key: "key", title: "Key" },
          { key: "value", title: "Value" },
        ],
        uniqueKey: "key",
        fetchData: fetchDataMock,
      });
      setTableData("table_with_string_unique_key", [
        { key: "key1", value: "value1" },
        { key: "key2", value: "value2" },
        { key: "key3", value: "value3" },
      ]);
      const table = getTable("table_with_string_unique_key");
      expect(table?.data?.size).toBe(3);
    });
  });

  describe("addRow", () => {
    it("should add rows", () => {
      const table = getTable(TABLE_NAME);
      expect(table?.data?.size).toBe(0);
      tableData.forEach((row) => {
        addRow(TABLE_NAME, row);
      });
      expect(table?.data?.size).toBe(5);
    });
    it("should not add row if table does not exist", () => {
      const table = getTable("non_existent_table");
      expect(table).toBeUndefined();
      addRow("non_existent_table", tableData[0] as TRow);
      expect(table).toBeUndefined();
    });
  });

  describe("removeRow", () => {
    it("should remove row", () => {
      const table = getTable(TABLE_NAME);
      setTableData(TABLE_NAME, tableData);
      expect(table?.data?.get("user/app1")).toBeDefined();
      expect(table?.data?.size).toBe(5);
      removeRow(TABLE_NAME, "user/app1");
      expect(table?.data?.size).toBe(4);
      expect(table?.data?.get("user/app1")).toBeUndefined();
    });
  });

  describe("clearTableData", () => {
    it("should clear table data", () => {
      const table = getTable(TABLE_NAME);
      setTableData(TABLE_NAME, tableData);
      expect(table?.data?.size).toBe(5);
      clearTableData(TABLE_NAME);
      expect(table?.data?.size).toBe(0);
    });
  });

  describe("getRows", () => {
    beforeEach(() => {
      const filter = getTableFilter(TABLE_NAME);
      filter?.filters.clear();
      filter?.sort.clear();
    });
    it("should return all rows when filters are not set", () => {
      setTableData(TABLE_NAME, tableData);
      expect(getRows(TABLE_NAME).length).toBe(5);
    });

    it("should return filtered rows", () => {
      setTableData(TABLE_NAME, tableData);
      addFilter(TABLE_NAME, "role", "admin");
      expect(getRows(TABLE_NAME).length).toBe(3);
    });

    it("should return filtered rows with multiple filters", () => {
      setTableData(TABLE_NAME, tableData);
      addFilter(TABLE_NAME, "role", "admin");
      addFilter(TABLE_NAME, "db_type", "memory");
      expect(getRows(TABLE_NAME).length).toBe(1);
    });

    it("should return empty array if no rows match filter", () => {
      setTableData(TABLE_NAME, tableData);
      addFilter(TABLE_NAME, "role", "non_existent_role");
      expect(getRows(TABLE_NAME).length).toBe(0);
    });

    it("should return sorted rows in desc order", () => {
      setTableData(TABLE_NAME, tableData);
      setSort(TABLE_NAME, "size", "desc");
      expect(getRows(TABLE_NAME)?.[0]?.[1].db).toBe("app3");
      expect(getRows(TABLE_NAME)?.[0]?.[1].owner).toBe("admin");
    });
    it("should return sorted rows in asc order", () => {
      setTableData(TABLE_NAME, tableData);
      setSort(TABLE_NAME, "size", "asc");
      expect(getRows(TABLE_NAME)?.[0]?.[1].db).toBe("app1");
      expect(getRows(TABLE_NAME)?.[0]?.[1].owner).toBe("admin");
    });
    it("should return sorted rows with multiple sort keys", () => {
      setTableData(TABLE_NAME, tableData);
      setSort(TABLE_NAME, "role", "asc");
      setSort(TABLE_NAME, "owner", "desc");
      expect(getRows(TABLE_NAME)?.[0]?.[1].db).toBe("app1");
      expect(getRows(TABLE_NAME)?.[0]?.[1].owner).toBe("user");
    });
    it("should return empty array if table doesn't exist", () => {
      expect(getRows("non_existent_table").length).toBe(0);
    });
  });
});
