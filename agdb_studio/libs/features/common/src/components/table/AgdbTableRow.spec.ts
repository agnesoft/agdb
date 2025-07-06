import { mount } from "@vue/test-utils";
import AgdbTableRow from "./AgdbTableRow.vue";
import { columnsMap, TABLE_NAME, tableConfig } from "../../tests/tableMocks";
import { describe, it, expect, vi } from "vitest";
import {
  INJECT_KEY_COLUMNS,
  INJECT_KEY_TABLE_NAME,
} from "@/composables/table/constants";
import { addTable } from "@/composables/table/tableConfig";

const { getDbUsers, fetchDbUsers, isDbRoleType } = vi.hoisted(() => {
  return {
    getDbUsers: vi.fn().mockReturnValue([]),
    fetchDbUsers: vi.fn().mockResolvedValue({ data: [] }),
    isDbRoleType: vi.fn().mockReturnValue(true),
  };
});

vi.mock("@/composables/db/dbUsersStore", () => {
  return {
    useDbUsersStore: () => {
      return {
        getDbUsers,
        fetchDbUsers,
        isDbRoleType,
      };
    },
  };
});

describe("TableRow", () => {
  const fetchDataMock = vi.fn();
  addTable({
    name: TABLE_NAME,
    columns: tableConfig,
    rowDetailsComponent: "DbDetails",
    fetchData: fetchDataMock,
  });

  it("should render", () => {
    const wrapper = mount(AgdbTableRow, {
      props: {
        columns: columnsMap,
        row: {
          role: "admin",
          owner: "admin",
          db: "app3",
          db_type: "file",
          size: 50,
          backup: 0,
        },
      },

      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columnsMap },
          [INJECT_KEY_TABLE_NAME]: { value: TABLE_NAME },
        },
      },
    });
    expect(wrapper.text()).toContain("admin");
    expect(wrapper.text()).toContain("app3");
  });

  it("should expand row", async () => {
    const wrapper = mount(AgdbTableRow, {
      props: {
        columns: columnsMap,
        row: {
          role: "admin",
          owner: "admin",
          db: "app3",
          db_type: "file",
          size: 50,
          backup: 0,
        },
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columnsMap },
          [INJECT_KEY_TABLE_NAME]: { value: TABLE_NAME },
        },
        stubs: {
          transitions: false,
          DbDetails: true,
        },
      },
    });

    expect(wrapper.find(".expanded-row").exists()).toBe(false);
    await wrapper.find(".expand-row").trigger("click");
    await wrapper.vm.$nextTick();
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".expanded-row").exists()).toBe(true);
  });

  it("should not render expand button if rowDetailsComponent is not set", () => {
    addTable({
      name: "table_without_row_details",
      columns: tableConfig,
      fetchData: fetchDataMock,
    });
    const wrapper = mount(AgdbTableRow, {
      props: {
        columns: columnsMap,
        row: {
          role: "admin",
          owner: "admin",
          db: "app3",
          db_type: "file",
          size: 50,
          backup: 0,
        },
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columnsMap },
          [INJECT_KEY_TABLE_NAME]: {
            value: "table_without_row_details",
          },
        },
        stubs: {
          transitions: false,
        },
      },
    });
    expect(wrapper.find(".expand-row").exists()).toBe(false);
  });
  it("should handle if tableKey is undefined", () => {
    const wrapper = mount(AgdbTableRow, {
      props: {
        columns: columnsMap,
        row: {
          role: "admin",
          owner: "admin",
          db: "app3",
          db_type: "file",
          size: 50,
          backup: 0,
        },
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columnsMap },
          [INJECT_KEY_TABLE_NAME]: undefined,
        },
        stubs: {
          transitions: false,
        },
      },
    });
    expect(wrapper.find(".expand-row").exists()).toBe(false);
  });
});
