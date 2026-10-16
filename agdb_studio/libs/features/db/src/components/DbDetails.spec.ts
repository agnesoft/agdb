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

const mockUsers = ref([
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
]);

vi.mock("../composables/dbDetails", () => {
  return {
    useDbDetails: () => {
      return {
        users: mockUsers,
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
    mockUsers.value = [
      { username: "testUser", role: "read" },
      { username: "testUser2", role: "write" },
      { username: "testUser3", role: "admin" },
    ];
  });

  it("should render users", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();
    expect(wrapper.find("header").text()).toContain("testOwner/testDb");
    const usernames = wrapper.findAll(".username");
    expect(usernames.length).toBe(3);
    expect(usernames[0]?.text()).toContain("testUser3");
    expect(usernames[1]?.text()).toContain("testUser2");
    expect(usernames[2]?.text()).toContain("testUser");
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

    const adminGroup = wrapper.findAll(".role-group")[0];
    const adminItems = adminGroup?.findAll(".user-item") ?? [];
    const ownerItem = adminItems.find((item) =>
      item.text().includes("testUser3"),
    );
    expect(ownerItem?.find(".remove-button").exists()).toBe(false);
  });

  it("should handle username click", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();
    const usernames = wrapper.findAll(".username");
    const readUser = usernames.find((u) => u.text().includes("testUser2"));
    expect(readUser).toBeDefined();
    await readUser!.trigger("click");
    await wrapper.vm.$nextTick();
    expect(handleUsernameClick).toHaveBeenCalledWith("testUser2", "write");
  });

  it("should group users by role", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();

    const headers = wrapper.findAll(".role-group-header");
    expect(headers.length).toBe(3);
    expect(headers[0]?.text()).toContain("Admin");
    expect(headers[0]?.text()).toContain("(1)");
    expect(headers[1]?.text()).toContain("Read/Write");
    expect(headers[1]?.text()).toContain("(1)");
    expect(headers[2]?.text()).toContain("Read Only");
    expect(headers[2]?.text()).toContain("(1)");
  });

  it("should display role summary counts", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();

    const summary = wrapper.find(".role-summary");
    expect(summary.exists()).toBe(true);
    const counts = wrapper.findAll(".role-count");
    expect(counts.length).toBe(3);
    expect(counts[0]?.text()).toContain("1 Admin");
    expect(counts[1]?.text()).toContain("1 Read/Write");
    expect(counts[2]?.text()).toContain("1 Read Only");
  });

  it("should filter users by search query", async () => {
    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();

    const input = wrapper.find(".search-input");
    await input.setValue("testUser2");
    await wrapper.vm.$nextTick();

    const usernames = wrapper.findAll(".username");
    expect(usernames.length).toBe(1);
    expect(usernames[0]?.text()).toContain("testUser2");
  });

  it("should show 'more' button when group has more than 10 users", async () => {
    const manyReaders = Array.from({ length: 15 }, (_, i) => ({
      username: `reader_${i}`,
      role: "read",
    }));
    mockUsers.value = [{ username: "admin1", role: "admin" }, ...manyReaders];

    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();

    const showMoreButtons = wrapper.findAll(".show-more-button");
    expect(showMoreButtons.length).toBe(1);
    expect(showMoreButtons[0]?.text()).toContain("+5 more");
  });

  it("should expand group when 'more' button is clicked", async () => {
    const manyReaders = Array.from({ length: 15 }, (_, i) => ({
      username: `reader_${i}`,
      role: "read",
    }));
    mockUsers.value = [{ username: "admin1", role: "admin" }, ...manyReaders];

    const wrapper = mount(DbDetails);
    await wrapper.vm.$nextTick();

    let readItems =
      wrapper.findAll(".role-group")[1]?.findAll(".user-item") ?? [];
    expect(readItems.length).toBe(10);

    await wrapper.find(".show-more-button").trigger("click");
    await wrapper.vm.$nextTick();

    readItems = wrapper.findAll(".role-group")[1]?.findAll(".user-item") ?? [];
    expect(readItems.length).toBe(15);

    const showMoreButton = wrapper.find(".show-more-button");
    expect(showMoreButton.text()).toBe("Show less");
  });
});
