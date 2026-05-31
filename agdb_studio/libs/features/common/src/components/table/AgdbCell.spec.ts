import { describe, beforeEach, vi, it, expect } from "vitest";
import { mount, shallowMount } from "@vue/test-utils";
import AgdbCell from "./AgdbCell.vue";
import {
  INJECT_KEY_COLUMNS,
  INJECT_KEY_ROW,
} from "../../composables/table/constants";
import { columnsMap } from "../../tests/tableMocks";

const TestIcon = {
  template: '<span data-testid="test-icon" />',
};

describe("AgdbCell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("should render the cell", () => {
    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "owner",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columnsMap },
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
        },
      },
    });

    expect(wrapper.find(".agdb-cell").exists()).toBe(true);
    expect(wrapper.find(".agdb-cell").text()).toBe("admin");
  });

  it("should render the cell with a formatter", () => {
    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "backup",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columnsMap },
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 123456,
            },
          },
        },
      },
    });

    expect(wrapper.find(".agdb-cell").exists()).toBe(true);
    expect(wrapper.find(".agdb-cell").text()).toBe("1");
  });

  it("should handle missing row data", () => {
    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "backup",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columnsMap },
          [INJECT_KEY_ROW]: undefined,
        },
      },
    });

    expect(wrapper.find(".agdb-cell").exists()).toBe(true);
    expect(wrapper.find(".agdb-cell").text()).toBe("0");
  });

  it("should display menu", async () => {
    const columns = new Map();
    columns.set("actions", {
      key: "actions",
      title: "Actions",
      actions: [
        {
          key: "backup",
          title: "Backup",
          action: () => {},
        },
      ],
    });
    const wrapper = shallowMount(AgdbCell, {
      props: {
        cellKey: "actions",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columns },
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
        },
      },
    });
    expect(wrapper.html()).toContain("agdb-cell-menu-stub");
  });
  it("should render boolean cell when it is true", () => {
    const columns = new Map();
    columns.set("backup", {
      key: "backup",
      title: "Backup",
      type: "boolean",
    });
    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "backup",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columns },
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: true,
            },
          },
        },
      },
    });

    expect(wrapper.find(".agdb-cell .positive-icon").exists()).toBe(true);
  });

  it("should render boolean cell when it is false", () => {
    const columns = new Map();
    columns.set("backup", {
      key: "backup",
      title: "Backup",
      type: "boolean",
    });
    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "backup",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columns },
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: false,
            },
          },
        },
      },
    });

    expect(wrapper.find(".agdb-cell .negative-icon").exists()).toBe(true);
  });

  it("should render static icon, title and class", () => {
    const columns = new Map();
    columns.set("owner", {
      key: "owner",
      title: "Owner",
      icon: TestIcon,
      iconTitle: "Owner icon",
      iconClass: "owner-icon",
    });

    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "owner",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columns },
          [INJECT_KEY_ROW]: {
            value: {
              owner: "admin",
            },
          },
        },
      },
    });

    const icon = wrapper.find('[data-testid="test-icon"]');
    expect(icon.exists()).toBe(true);
    expect(icon.classes()).toContain("owner-icon");
    expect(icon.attributes("title")).toBe("Owner icon");
  });

  it("should render icon using resolvers", () => {
    const columns = new Map();
    columns.set("owner", {
      key: "owner",
      title: "Owner",
      iconResolver: (row: { owner: string }) =>
        row.owner === "admin" ? TestIcon : undefined,
      iconTitleResolver: (row: { owner: string }) =>
        row.owner === "admin" ? "Admin icon" : undefined,
      iconClassResolver: (row: { owner: string }) =>
        row.owner === "admin" ? "admin-icon" : undefined,
    });

    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "owner",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columns },
          [INJECT_KEY_ROW]: {
            value: {
              owner: "admin",
            },
          },
        },
      },
    });

    const icon = wrapper.find('[data-testid="test-icon"]');
    expect(icon.exists()).toBe(true);
    expect(icon.classes()).toContain("admin-icon");
    expect(icon.attributes("title")).toBe("Admin icon");
  });

  it("should not render icon when resolver returns undefined", () => {
    const columns = new Map();
    columns.set("owner", {
      key: "owner",
      title: "Owner",
      iconResolver: () => undefined,
      iconTitleResolver: () => "hidden",
      iconClassResolver: () => "hidden",
    });

    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "owner",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columns },
          [INJECT_KEY_ROW]: {
            value: {
              owner: "user",
            },
          },
        },
      },
    });

    expect(wrapper.find('[data-testid="test-icon"]').exists()).toBe(false);
  });

  it("should render icon without title and class when optional fields are missing", () => {
    const columns = new Map();
    columns.set("owner", {
      key: "owner",
      title: "Owner",
      icon: TestIcon,
    });

    const wrapper = mount(AgdbCell, {
      props: {
        cellKey: "owner",
      },
      global: {
        provide: {
          [INJECT_KEY_COLUMNS]: { value: columns },
          [INJECT_KEY_ROW]: {
            value: {
              owner: "admin",
            },
          },
        },
      },
    });

    const icon = wrapper.find('[data-testid="test-icon"]');
    expect(icon.exists()).toBe(true);
    expect(icon.attributes("title")).toBeUndefined();
    expect(icon.classes()).toEqual([]);
  });
});
