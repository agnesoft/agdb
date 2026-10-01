import { vi, describe, it, beforeEach, expect } from "vitest";
import UserAddForm from "./UserAddForm.vue";
import { mount } from "@vue/test-utils";

const { addUser, fetchUsers } = vi.hoisted(() => {
  return {
    addUser: vi.fn(),
    fetchUsers: vi.fn(),
  };
});

vi.mock("../composables/userStore", () => {
  return {
    useUserStore: () => {
      return {
        addUser,
        fetchUsers,
      };
    },
  };
});

describe("UserAddForm", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should add a user when user submits", async () => {
    addUser.mockResolvedValueOnce(true);
    expect(addUser).not.toHaveBeenCalled();
    const wrapper = mount(UserAddForm);
    await wrapper.find("input#username").setValue("test_user");
    await wrapper.find("input#password").setValue("test_password");
    await wrapper.find("form").trigger("submit");
    await wrapper.vm.$nextTick();
    expect(addUser).toHaveBeenCalledOnce();
  });

  it("should add a user when user clicks submit button", async () => {
    addUser.mockResolvedValueOnce(true);
    expect(addUser).not.toHaveBeenCalled();
    const wrapper = mount(UserAddForm);
    await wrapper.find("input#username").setValue("test_user");
    await wrapper.find("input#password").setValue("test_password");
    await wrapper.find("form").trigger("submit");
    await wrapper.vm.$nextTick();
    expect(addUser).toHaveBeenCalledOnce();
  });

  it("should not add user when fields are empty", async () => {
    const wrapper = mount(UserAddForm);
    await wrapper.find("form").trigger("submit");
    await wrapper.vm.$nextTick();
    expect(addUser).not.toHaveBeenCalled();
  });

  it("should handle addUser failure", async () => {
    addUser.mockRejectedValueOnce(new Error("fail"));
    const wrapper = mount(UserAddForm);
    await wrapper.find("input#username").setValue("test_user");
    await wrapper.find("input#password").setValue("test_password");
    await wrapper.find("form").trigger("submit");
    await wrapper.vm.$nextTick();
    expect(addUser).toHaveBeenCalledOnce();
  });
});
