import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { ref } from "vue";

const { store, push } = vi.hoisted(() => ({
  store: {
    dashboards: undefined as unknown as import("vue").Ref<unknown[]>,
    fetchDashboards: vi.fn(),
    createDashboard: vi.fn(),
    renameDashboard: vi.fn(),
    removeDashboard: vi.fn(),
  },
  push: vi.fn(),
}));
vi.mock("../composables/dashboardStore", () => ({
  useDashboardStore: () => store,
}));
vi.mock("vue-router", () => ({ useRouter: () => ({ push }) }));

import DashboardList from "./DashboardList.vue";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";

const { modal, handleConfirm, closeModal } = useModal();
const { setInputValue } = useContentInputs();

const mountList = async () => {
  const wrapper = mount(DashboardList, {
    global: {
      stubs: {
        RouterLink: { props: ["to"], template: '<a class="link"><slot /></a>' },
      },
    },
  });
  await flushPromises();
  return wrapper;
};

describe("DashboardList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closeModal();
    store.dashboards = ref([
      { name: "ops", updated: 1_700_000_000 },
      { name: "never", updated: 0 },
    ]);
    store.fetchDashboards.mockResolvedValue(undefined);
  });

  it("lists dashboards with links and dates", async () => {
    const wrapper = await mountList();
    const links = wrapper.findAll(".link");
    expect(links.map((l) => l.text())).toEqual(["ops", "never"]);
    expect(wrapper.findAll(".updated")[0]!.text()).not.toBe("");
    expect(wrapper.findAll(".updated")[1]!.text()).toBe("");
  });

  it("shows an empty state, also when loading fails", async () => {
    store.dashboards.value = [];
    store.fetchDashboards.mockRejectedValueOnce(new Error("offline"));
    const wrapper = await mountList();
    expect(wrapper.find('[data-testid="dashboards-empty"]').exists()).toBe(
      true,
    );
  });

  it("creates a dashboard and opens it", async () => {
    const wrapper = await mountList();
    await wrapper.find('[data-testid="dashboard-create"]').trigger("click");
    expect(modal.header).toBe("New dashboard");
    const rule = modal.content[0]!.input!.rules![0]!;
    expect(rule("list")).toBeDefined();
    expect(rule(undefined as unknown as string)).toBeDefined();
    setInputValue(KEY_MODAL, "name", " sales ");
    await handleConfirm();
    await flushPromises();
    expect(store.createDashboard).toHaveBeenCalledWith("sales");
    expect(push).toHaveBeenCalledWith({
      name: "dashboard",
      params: { name: "sales" },
    });
  });

  it("renames and deletes dashboards", async () => {
    const wrapper = await mountList();
    await wrapper.find('[data-testid="dashboard-rename"]').trigger("click");
    expect(modal.content[0]!.input!.value).toBe("ops");
    setInputValue(KEY_MODAL, "name", "sales");
    await handleConfirm();
    await flushPromises();
    expect(store.renameDashboard).toHaveBeenCalledWith("ops", "sales");

    await wrapper.find('[data-testid="dashboard-remove"]').trigger("click");
    expect(modal.header).toBe("Delete dashboard");
    await handleConfirm();
    await flushPromises();
    expect(store.removeDashboard).toHaveBeenCalledWith("ops");
  });

  it("treats a missing name input as empty", async () => {
    const wrapper = await mountList();
    await wrapper.find('[data-testid="dashboard-rename"]').trigger("click");
    useContentInputs().getContentInputs(KEY_MODAL)?.delete("name");
    await handleConfirm();
    await flushPromises();
    expect(store.renameDashboard).toHaveBeenCalledWith("ops", "");
  });
});
