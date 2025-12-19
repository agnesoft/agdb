import { mount } from "@vue/test-utils";
import { describe, beforeEach, vi, it, expect } from "vitest";
import DbDetails from "./DbDetails.vue";
import { ref } from "vue";

const {
  fetchDbUsers,
  isDbRoleType,
  handleRemoveUser,
  handleAddUser,
  isOwner,
  handleUsernameClick,
} = vi.hoisted(() => {
  return {
    fetchDbUsers: vi.fn().mockResolvedValue({ data: [] }),
    isDbRoleType: vi.fn().mockReturnValue(true),
    handleRemoveUser: vi.fn(),
    handleAddUser: vi.fn(),
    isOwner: vi.fn().mockImplementation((name: string) => name === "testUser3"),
    handleUsernameClick: vi.fn(),
  };
});

vi.mock("../composables/dbUsersStore", () => {
  return {
    useDbUsersStore: () => {
      return {
        fetchDbUsers,
        isDbRoleType,
      };
    },
  };
});

const canEditUsers = ref(true);

vi.mock("../composables/dbDetails", () => {
  return {
    useDbDetails: () => {
      return {
        users: ref([
          {
            username: "testUser",
            role: "read",
          },
          {
            username: "testUser2",
            role: "write",
          },
          {
            username: "testUser3",
            role: "admin",
          },
        ]),
        dbName: ref("testOwner/testDb"),
        canEditUsers: canEditUsers,
        handleRemoveUser,
        handleAddUser,
        isOwner,
        handleUsernameClick,
      };
    },
  };
});

describe("DbDetails", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    canEditUsers.value = true;
  });
  it("should render users", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();
    expect(wrapper.find("header").text()).toContain("testOwner/testDb");
    const usernames = wrapper.findAll(".username");
    expect(usernames.length).toBe(3);
    expect(usernames[0]?.text()).toContain("testUser");
    expect(usernames[1]?.text()).toContain("testUser2");
    expect(usernames[2]?.text()).toContain("testUser3");

    const roles = wrapper.findAll(".role");
    expect(roles.length).toBe(3);
    expect(roles[0]?.text()).toContain("(R)");
    expect(roles[1]?.text()).toContain("(W)");
    expect(roles[2]?.text()).toContain("(A)");
  });

  it("should add a user", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();
    await wrapper.find(".add-button").trigger("click");
    await wrapper.vm.$nextTick();

    expect(handleAddUser).toHaveBeenCalled();
  });

  it("should remove a user", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();
    await wrapper.find(".remove-button").trigger("click");
    await wrapper.vm.$nextTick();

    expect(handleRemoveUser).toHaveBeenCalled();
  });

  it("should not render add button if not admin", async () => {
    canEditUsers.value = false;
    const wrapper = mount(DbDetails, {
      props: {
        row: {
          role: "read",
        },
      },
    });

    await wrapper.vm.$nextTick();
    const addButton = wrapper.find(".add-button");
    expect(addButton.exists()).toBe(false);
  });

  it("should not render remove button if not admin", async () => {
    canEditUsers.value = false;
    const wrapper = mount(DbDetails, {
      props: {
        row: {
          role: "read",
        },
      },
    });

    await wrapper.vm.$nextTick();
    const removeButton = wrapper.find(".remove-button");
    expect(removeButton.exists()).toBe(false);
  });

  it("should not render remove button if user is owner", async () => {
    const wrapper = mount(DbDetails, {
      props: {
        row: {
          owner: "testUser3",
          role: "admin",
          db: "testDb",
        },
      },
    });

    await wrapper.vm.$nextTick();
    const items = wrapper.findAll(".user-item");
    expect(items.length).toBe(3);
    expect(items[0]?.find(".remove-button").exists()).toBe(true);
    expect(items[2]?.find(".remove-button").exists()).toBe(false);
  });

  it("should handle username click", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();
    const usernames = wrapper.findAll(".username");
    await usernames[0]?.trigger("click");
    await wrapper.vm.$nextTick();
    expect(handleUsernameClick).toHaveBeenCalledWith("testUser", "read");
  });
});
