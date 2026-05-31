import { vi, describe, it, beforeEach, expect } from "vitest";
import UserTable from "./UserTable.vue";
import { mount, shallowMount } from "@vue/test-utils";
import { ref, nextTick } from "vue";

const users = ref([
  {
    username: "admin_user",
    admin: true,
    login: false,
  },
  {
    username: "test_user",
    admin: false,
    login: false,
  },
]);

vi.mock("../composables/userStore", () => {
  return {
    useUserStore: () => {
      return { users };
    },
  };
});

describe("UserTable", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders", () => {
    const wrapper = shallowMount(UserTable);
    expect(wrapper.exists()).toBe(true);
  });
  it("should render message when no users", () => {
    users.value = [];
    const wrapper = mount(UserTable);
    // wait for Vue to flush reactivity
    return nextTick().then(() => {
      expect(wrapper.text()).toContain("No users found");
    });
  });

  it("renders admin user first with crown next to username", async () => {
    users.value = [
      {
        username: "admin_user",
        admin: true,
        login: true,
      },
      {
        username: "test_user",
        admin: false,
        login: false,
      },
    ];

    const wrapper = mount(UserTable);
    await nextTick();

    const usernameCells = wrapper.findAll(
      '[data-testid="table-cell-username"]',
    );
    expect(usernameCells).toHaveLength(2);

    expect(usernameCells[0]?.text()).toContain("admin_user");
    expect(usernameCells[0]?.find(".crown-icon").exists()).toBe(true);

    expect(usernameCells[1]?.text()).toContain("test_user");
    expect(usernameCells[1]?.find(".crown-icon").exists()).toBe(false);
  });
});
