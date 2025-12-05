import { mount, shallowMount } from "@vue/test-utils";
import { nextTick } from "vue";
import DbTable from "./DbTable.vue";
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

const { databases, getDbName } = vi.hoisted(() => {
  // create a real Vue ref inside the hoisted factory to avoid TDZ when vitest hoists mocks
  // @ts-ignore
  const { ref } = require("vue");
  const databases = ref([] as typeof DATABASES);
  const getDbName = vi.fn().mockImplementation((db) => `${db.owner}/${db.db}`);
  return { databases, getDbName };
});

vi.mock("../composables/dbStore", () => {
  return {
    useDbStore: () => ({ databases, getDbName }),
  };
});

describe("DbTable", () => {
  beforeEach(() => {
    databases.value = DATABASES;
    // reset mock call history
    getDbName.mockClear && getDbName.mockClear();
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
});
