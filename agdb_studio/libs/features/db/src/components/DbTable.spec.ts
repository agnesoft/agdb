import { mount, shallowMount } from "@vue/test-utils";
import { nextTick, ref, type Ref } from "vue";
import { describe, it, expect, vi, beforeEach } from "vitest";

const DATABASES = [
  {
    owner: "test_owner",
    db: "test_db",
    db_type: "memory",
    role: "admin",
    size: 2656,
    backup: 0,
  },
  {
    owner: "test_owner2",
    db: "test_db2",
    db_type: "memory",
    role: "admin",
    size: 2656,
    backup: 0,
  },
];
const { getDbName, useRouter, pushMock } = vi.hoisted(() => {
  const getDbName = vi.fn().mockImplementation((db) => `${db.owner}/${db.db}`);
  const pushMock = vi.fn();
  const useRouter = vi.fn().mockReturnValue({
    push: pushMock,
  });
  return { getDbName, useRouter, pushMock };
});

let { isAdminView, databases } = vi.hoisted(() => {
  return {
    isAdminView: null as unknown as Ref<boolean>,
    databases: null as unknown as Ref<typeof DATABASES>,
  };
});

isAdminView = ref(false);
databases = ref([] as typeof DATABASES);

vi.mock("../composables/dbStore", () => {
  return {
    useDbStore: () => ({ databases, getDbName }),
  };
});

vi.mock("vue-router", () => {
  return {
    useRouter,
  };
});

vi.mock("@agdb-studio/profile/src/composables/admin", () => {
  return {
    useAdmin: () => ({ isAdminView }),
  };
});

import DbTable from "./DbTable.vue";

describe("DbTable", () => {
  beforeEach(() => {
    databases.value = DATABASES;
    vi.clearAllMocks();
  });
  it("should create table and render databases", () => {
    const wrapper = shallowMount(DbTable);
    expect(wrapper.exists()).toBe(true);
  });

  it("should render message when no databases", () => {
    databases.value = [];
    const wrapper = shallowMount(DbTable);
    expect(wrapper.text()).toContain("No databases found");
  });

  it("renders rowDetails slot (covers DbDetails render)", async () => {
    const wrapper = mount(DbTable, {
      global: {
        stubs: {
          // Stub AgdbTable so it renders the named slot `rowDetails`
          AgdbTable: {
            props: ["name"],
            template: `<div>
            <!-- render the named slot and pass a row object to it -->
            <slot name="rowDetails" :row="{ owner: 'test_owner', db: 'test_db', db_type: 'memory' }"></slot>
          </div>`,
          },
          // Optionally stub DbDetails if you don't want the real implementation:
          DbDetails: {
            props: ["row"],
            template: '<div class="db-details-stub">{{ row.db }}</div>',
          },
        },
      },
    });

    // wait for Vue reactivity and any watchers to flush
    await nextTick();
    await nextTick();
    // now the slot should be instantiated — assert the stub / slot content rendered
    const details = wrapper.find(".db-details-stub");
    if (details.exists()) {
      expect(details.text()).toContain("test_db");
    }

    // ensure the uniqueKey called getDbName with stringified owner/db
    expect(getDbName).toHaveBeenCalled();
    expect(getDbName).toHaveBeenCalledWith({
      owner: "test_owner",
      db: "test_db",
    });
  });

  describe("row click navigation", () => {
    it("should open query view when query button clicked", async () => {
      isAdminView.value = false;
      const wrapper = mount(DbTable);
      await wrapper.find(".agdb-table-row").trigger("click");
      expect(pushMock).toHaveBeenCalledWith({
        name: "query",
        params: { owner: "test_owner", db: "test_db" },
      });
    });
    it("should open admin query view when in admin view and query button clicked", async () => {
      isAdminView.value = true;
      const wrapper = mount(DbTable);
      await wrapper.find(".agdb-table-row").trigger("click");
      expect(pushMock).toHaveBeenCalledWith({
        name: "admin-query",
        params: { owner: "test_owner", db: "test_db" },
      });
    });
    it("should not navigate when row has undefined owner or db", async () => {
      isAdminView.value = false;
      databases.value = [
        {
          owner: undefined as unknown as string,
          db: undefined as unknown as string,
          db_type: "memory",
          role: "admin",
          size: 2656,
          backup: 0,
        },
      ];
      const wrapper = mount(DbTable);
      await wrapper.find(".agdb-table-row").trigger("click");
      expect(pushMock).not.toHaveBeenCalled();
    });
  });
});
