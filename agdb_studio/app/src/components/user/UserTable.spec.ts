import { vi, describe, it, beforeEach, expect } from "vitest";
import UserTable from "./UserTable.vue";
import { mount, shallowMount } from "@vue/test-utils";

const { users } = vi.hoisted(() => {
  return {
    users: {
      value: [
        {
          username: "test_user",
          admin: false,
          login: false,
        },
        {
          username: "test_user2",
          admin: false,
          login: false,
        },
      ],
    },
  };
});

vi.mock("@/composables/user/userStore", () => {
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
    expect(wrapper.text()).toContain("No users found");
  });
});
