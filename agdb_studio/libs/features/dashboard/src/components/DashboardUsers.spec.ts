import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";

const { client, addNotification, store } = vi.hoisted(() => ({
  store: { fetchDashboards: vi.fn(), leaveDashboard: vi.fn() },
  client: {
    value: {
      dashboard_user_list: vi.fn(),
      dashboard_user_add: vi.fn(),
      dashboard_user_remove: vi.fn(),
      admin_dashboard_user_list: vi.fn(),
      admin_dashboard_user_add: vi.fn(),
      admin_dashboard_user_remove: vi.fn(),
    },
  },
  addNotification: vi.fn(),
}));
vi.mock("../composables/dashboardStore", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../composables/dashboardStore")>()),
  useDashboardStore: () => store,
}));
vi.mock("@agdb-studio/api/src/api", () => ({ checkClient: vi.fn(), client }));
vi.mock("@agdb-studio/auth/src/account", async () => {
  const { ref } = await import("vue");
  return { useAccount: () => ({ username: ref("me") }) };
});
vi.mock("@agdb-studio/db/src/composables/dbUsersStore", () => ({
  useDbUsersStore: () => ({
    isDbRoleType: (role: string) => ["admin", "write", "read"].includes(role),
  }),
}));
vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification,
}));

import type { Ref } from "vue";
import { useAdmin } from "@agdb-studio/router/src/admin";
import DashboardUsers from "./DashboardUsers.vue";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";

const { modal, handleConfirm, closeModal, modalIsVisible } = useModal();
const { setInputValue } = useContentInputs();

const key = { owner: "me", name: "ops" };
const shared = { owner: "ann", name: "ops" };

const mountUsers = async (
  role: "admin" | "write" | "read" = "admin",
  dashboard = key,
) => {
  const wrapper = mount(DashboardUsers, { props: { ...dashboard, role } });
  await flushPromises();
  return wrapper;
};

const usernames = (wrapper: Awaited<ReturnType<typeof mountUsers>>) =>
  wrapper.findAll(".username").map((u) => u.text());

describe("DashboardUsers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closeModal();
    client.value.dashboard_user_list.mockResolvedValue({
      data: [
        { username: "me", role: "admin" },
        { username: "ann", role: "read" },
      ],
    });
  });

  it("lists the users by role", async () => {
    const wrapper = await mountUsers();
    expect(client.value.dashboard_user_list).toHaveBeenCalledWith(key);
    expect(wrapper.find("h2").text()).toBe("Dashboard: me/ops");
    expect(usernames(wrapper)).toEqual(["me", "ann"]);
    // The owner can't be removed.
    expect(wrapper.findAll(".remove-button")).toHaveLength(1);
  });

  it("lets only admins change users", async () => {
    const wrapper = await mountUsers("write");
    expect(wrapper.find(".add-button").exists()).toBe(false);
    expect(wrapper.find(".remove-button").exists()).toBe(false);
    await wrapper.findAll(".username")[1]!.trigger("click");
    expect(modalIsVisible.value).toBe(false);
  });

  it("survives a failed load", async () => {
    client.value.dashboard_user_list.mockRejectedValueOnce(new Error("403"));
    const wrapper = await mountUsers();
    expect(usernames(wrapper)).toEqual([]);
  });

  it("adds a user", async () => {
    const wrapper = await mountUsers();
    await wrapper.find(".add-button").trigger("click");
    expect(modal.header).toBe("Share dashboard");
    setInputValue(KEY_MODAL, "username", " bob ");
    setInputValue(KEY_MODAL, "role", "write");
    await handleConfirm();
    await flushPromises();
    expect(client.value.dashboard_user_add).toHaveBeenCalledWith({
      ...key,
      username: "bob",
      db_role: "write",
    });
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: "success" }),
    );
    expect(client.value.dashboard_user_list).toHaveBeenCalledTimes(2);
    expect(store.fetchDashboards).not.toHaveBeenCalled();
  });

  it("refreshes the dashboards after changing the own role", async () => {
    client.value.dashboard_user_list.mockResolvedValue({
      data: [
        { username: "ann", role: "admin" },
        { username: "me", role: "admin" },
      ],
    });
    const wrapper = await mountUsers("admin", shared);
    await wrapper.findAll(".username")[1]!.trigger("click");
    setInputValue(KEY_MODAL, "role", "read");
    await handleConfirm();
    await flushPromises();
    expect(client.value.dashboard_user_add).toHaveBeenCalledWith({
      ...shared,
      username: "me",
      db_role: "read",
    });
    expect(store.fetchDashboards).toHaveBeenCalled();
  });

  it("leaves through the store when removing oneself", async () => {
    client.value.dashboard_user_list.mockResolvedValue({
      data: [
        { username: "ann", role: "admin" },
        { username: "me", role: "admin" },
      ],
    });
    const wrapper = await mountUsers("admin", shared);
    await wrapper.find(".remove-button").trigger("click");
    await handleConfirm();
    await flushPromises();
    expect(store.leaveDashboard).toHaveBeenCalledWith(shared);
    expect(client.value.dashboard_user_remove).not.toHaveBeenCalled();
    expect(client.value.dashboard_user_list).toHaveBeenCalledTimes(1);
  });

  it("keeps the dialog open without a username or a valid role", async () => {
    const wrapper = await mountUsers();
    await wrapper.find(".add-button").trigger("click");
    setInputValue(KEY_MODAL, "username", "bob");
    setInputValue(KEY_MODAL, "role", "owner");
    await handleConfirm();
    await flushPromises();
    useContentInputs().getContentInputs(KEY_MODAL)?.delete("username");
    useContentInputs().getContentInputs(KEY_MODAL)?.delete("role");
    await handleConfirm();
    await flushPromises();
    expect(client.value.dashboard_user_add).not.toHaveBeenCalled();
    expect(modalIsVisible.value).toBe(true);
  });

  it("changes the role of a user but not of the owner", async () => {
    const wrapper = await mountUsers();
    await wrapper.findAll(".username")[0]!.trigger("click");
    expect(modalIsVisible.value).toBe(false);
    await wrapper.findAll(".username")[1]!.trigger("click");
    expect(modal.content[1]!.input!.value).toBe("ann");
    expect(modal.content[2]!.input!.value).toBe("read");
  });

  it("removes a user", async () => {
    const wrapper = await mountUsers();
    await wrapper.find(".remove-button").trigger("click");
    expect(modal.header).toBe("Remove user");
    await handleConfirm();
    await flushPromises();
    expect(client.value.dashboard_user_remove).toHaveBeenCalledWith({
      ...key,
      username: "ann",
    });
    expect(client.value.dashboard_user_list).toHaveBeenCalledTimes(2);
  });

  it("manages users through the admin routes in the admin view", async () => {
    const { isAdmin, isAdminView } = useAdmin();
    (isAdmin as Ref<boolean>).value = true;
    (isAdminView as Ref<boolean>).value = true;
    client.value.admin_dashboard_user_list.mockResolvedValue({
      data: [
        { username: "me", role: "admin" },
        { username: "ann", role: "read" },
      ],
    });
    const wrapper = await mountUsers();
    expect(usernames(wrapper)).toContain("ann");
    expect(client.value.dashboard_user_list).not.toHaveBeenCalled();

    await wrapper.find(".add-button").trigger("click");
    setInputValue(KEY_MODAL, "username", "bob");
    setInputValue(KEY_MODAL, "role", "read");
    await handleConfirm();
    await flushPromises();
    expect(client.value.admin_dashboard_user_add).toHaveBeenCalledWith({
      ...key,
      username: "bob",
      db_role: "read",
    });

    await wrapper.find(".remove-button").trigger("click");
    await handleConfirm();
    await flushPromises();
    expect(client.value.admin_dashboard_user_remove).toHaveBeenCalledWith({
      ...key,
      username: "ann",
    });
    expect(client.value.dashboard_user_add).not.toHaveBeenCalled();
    expect(client.value.dashboard_user_remove).not.toHaveBeenCalled();
  });
});
