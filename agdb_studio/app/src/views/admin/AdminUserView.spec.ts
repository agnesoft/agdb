import { vi, describe, it, beforeEach, expect } from "vitest";
import AdminUserView from "./AdminUserView.vue";
import { mount, shallowMount, flushPromises } from "@vue/test-utils";

const { users, fetchUsers, logoutAllUsers } = vi.hoisted(() => {
  return {
    users: {
      value: [
        {
          username: "test_user",
          admin: false,
          login: false,
          sessions: [],
        },
        {
          username: "test_user2",
          admin: false,
          login: false,
          sessions: [],
        },
      ],
    },
    fetchUsers: vi.fn().mockResolvedValue(undefined),
    logoutAllUsers: vi.fn(),
  };
});

vi.mock("@agdb-studio/user/src/composables/userStore", () => {
  return {
    useUserStore: () => {
      return { users, fetchUsers };
    },
  };
});

vi.mock(
  "@agdb-studio/user/src/composables/userConfig",
  async (importOriginal) => {
    const original =
      await importOriginal<
        typeof import("@agdb-studio/user/src/composables/userConfig")
      >();
    return { ...original, logoutAllUsers };
  },
);

describe("AdminUserView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    fetchUsers.mockResolvedValue(undefined);
  });
  it("renders the admin user view", async () => {
    const wrapper = shallowMount(AdminUserView);
    await flushPromises();
    expect(wrapper.html()).toContain("user-table");
    expect(wrapper.html()).toContain("user-add-form");
  });

  it("should show spinner while loading", () => {
    fetchUsers.mockReturnValue(new Promise(() => {}));
    const wrapper = shallowMount(AdminUserView);
    expect(wrapper.html()).toContain("spinner-icon-stub");
    expect(wrapper.html()).not.toContain("user-table");
  });

  it("should refresh users when user clicks refresh button", async () => {
    expect(fetchUsers).not.toHaveBeenCalled();
    const wrapper = mount(AdminUserView);
    await flushPromises();
    expect(fetchUsers).toHaveBeenCalledTimes(1);

    await wrapper.find("button.refresh").trigger("click");
    await flushPromises();
    expect(fetchUsers).toHaveBeenCalledTimes(2);
  });

  it("should fetch users when the page view loads", () => {
    expect(fetchUsers).not.toHaveBeenCalled();
    mount(AdminUserView);
    expect(fetchUsers).toHaveBeenCalledOnce();
  });

  it("should render message when no users", async () => {
    users.value = [];
    const wrapper = mount(AdminUserView);
    await flushPromises();
    expect(wrapper.text()).toContain("No users found");
  });
  it("should clear loading state when fetch fails", async () => {
    fetchUsers.mockRejectedValueOnce(new Error("network error"));
    const wrapper = shallowMount(AdminUserView);
    await flushPromises();
    expect(wrapper.html()).not.toContain("spinner-icon-stub");
  });

  it("renders logout all button and calls logoutAllUsers on click", async () => {
    const wrapper = shallowMount(AdminUserView);
    await flushPromises();
    const button = wrapper.find('[data-testid="logout-all-button"]');
    expect(button.exists()).toBe(true);
    expect(button.text()).toContain("Logout All");
    await button.trigger("click");
    expect(logoutAllUsers).toHaveBeenCalledOnce();
  });
});
