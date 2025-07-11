import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import ProfileMenu from "./ProfileMenu.vue";
import { user_change_password } from "@agdb-studio/testing/mocks/apiMock";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";

const { logout } = vi.hoisted(() => {
  return {
    logout: vi.fn(),
  };
});

vi.mock("@agdb-studio/auth/src/auth", () => {
  return {
    useAuth: () => ({
      logout,
    }),
  };
});

vi.mock("@agdb-studio/auth/src/account", () => {
  return {
    useAccount: () => ({
      username: "testUser",
      admin: { value: false },
    }),
  };
});

const { modalIsVisible, closeModal, handleConfirm, onConfirm } = useModal();
const { setInputValue } = useContentInputs();

describe("ProfileDropdown", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closeModal();
  });
  it("renders the user actions", () => {
    const wrapper = mount(ProfileMenu);
    expect(wrapper.text()).toContain("Change password");
    expect(wrapper.text()).toContain("Logout");
  });

  it("should logout on click", async () => {
    const wrapper = mount(ProfileMenu);
    const logoutElement = wrapper.find(".menu-item[data-key=logout]");
    logoutElement.trigger("click");
    await wrapper.vm.$nextTick();
    expect(modalIsVisible.value).toBe(true);
    handleConfirm();
    await wrapper.vm.$nextTick();
    expect(logout).toHaveBeenCalled();
  });

  it("should open the change password modal on click", async () => {
    const wrapper = mount(ProfileMenu);
    const changePasswordElement = wrapper.find(
      ".menu-item[data-key=change-password]",
    );
    expect(modalIsVisible.value).toBe(false);
    changePasswordElement.trigger("click");
    await wrapper.vm.$nextTick();
    expect(modalIsVisible.value).toBe(true);
  });

  it("should change password on confirm", async () => {
    const wrapper = mount(ProfileMenu);
    const changePasswordElement = wrapper.find(
      ".menu-item[data-key=change-password]",
    );
    changePasswordElement.trigger("click");
    await wrapper.vm.$nextTick();
    expect(modalIsVisible.value).toBe(true);
    setInputValue(KEY_MODAL, "currentPassword", "test");
    setInputValue(KEY_MODAL, "newPassword", "testtest");
    setInputValue(KEY_MODAL, "confirmNewPassword", "testtest");

    handleConfirm();
    await wrapper.vm.$nextTick();
    expect(user_change_password).toHaveBeenCalled();
  });

  it("should check the length of the new password", async () => {
    const wrapper = mount(ProfileMenu);
    const changePasswordElement = wrapper.find(
      ".menu-item[data-key=change-password]",
    );
    changePasswordElement.trigger("click");
    await wrapper.vm.$nextTick();
    expect(modalIsVisible.value).toBe(true);

    setInputValue(KEY_MODAL, "currentPassword", "test");
    setInputValue(KEY_MODAL, "newPassword", "short");
    setInputValue(KEY_MODAL, "confirmNewPassword", "short");
    handleConfirm();
    await wrapper.vm.$nextTick();
    expect(user_change_password).not.toHaveBeenCalled();
    expect(modalIsVisible.value).toBe(true);
  });

  it("should check the match of the new password", async () => {
    const wrapper = mount(ProfileMenu);
    const changePasswordElement = wrapper.find(
      ".menu-item[data-key=change-password]",
    );
    changePasswordElement.trigger("click");
    await wrapper.vm.$nextTick();
    expect(modalIsVisible.value).toBe(true);

    setInputValue(KEY_MODAL, "currentPassword", "test");
    setInputValue(KEY_MODAL, "newPassword", "testtest");
    setInputValue(KEY_MODAL, "confirmNewPassword", "testtest2");
    handleConfirm();
    await wrapper.vm.$nextTick();
    expect(user_change_password).not.toHaveBeenCalled();
    expect(modalIsVisible.value).toBe(true);
  });

  it("should check the match of new password also in the confirm", async () => {
    const wrapper = mount(ProfileMenu);
    const changePasswordElement = wrapper.find(
      ".menu-item[data-key=change-password]",
    );
    changePasswordElement.trigger("click");
    await wrapper.vm.$nextTick();
    expect(modalIsVisible.value).toBe(true);

    setInputValue(KEY_MODAL, "currentPassword", "test");
    setInputValue(KEY_MODAL, "newPassword", "testtest");
    setInputValue(KEY_MODAL, "confirmNewPassword", "testtest2");
    onConfirm.value?.();
    await wrapper.vm.$nextTick();
    expect(user_change_password).not.toHaveBeenCalled();
  });
});
