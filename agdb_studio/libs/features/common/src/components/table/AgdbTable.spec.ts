import { mount } from "@vue/test-utils";
import AgdbTable from "./AgdbTable.vue";
import { addTable, clearTables } from "@/composables/table/tableConfig";
import { setTableData } from "@/composables/table/tableData";
import { TABLE_NAME, tableConfig, tableData } from "@/tests/tableMocks";
import { describe, beforeEach, it, expect, vi } from "vitest";

describe("AgdbTable", () => {
  beforeEach(() => {
    clearTables();
  });
  it("should render for correct data", () => {
    addTable({
      name: TABLE_NAME,
      columns: tableConfig,
      fetchData: vi.fn(),
    });
    setTableData(TABLE_NAME, tableData);

    const wrapper = mount(AgdbTable, {
      props: {
        name: TABLE_NAME,
      },
    });
    expect(wrapper.findAll(".agdb-table-row").length).toBe(tableData.length);
  });
  it("should not render rows when table doesn't exist", () => {
    const wrapper = mount(AgdbTable, {
      props: {
        name: TABLE_NAME,
      },
    });
    expect(wrapper.findAll(".agdb-table-row").length).toBe(0);
  });
});
