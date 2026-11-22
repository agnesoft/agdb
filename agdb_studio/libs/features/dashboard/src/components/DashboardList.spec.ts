import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { computed, ref, type Ref } from "vue";
import type { DashboardSummary } from "@agnesoft/agdb_api";

const { store, push } = vi.hoisted(() => ({
  store: {
    dashboards: undefined as unknown as import("vue").Ref<DashboardSummary[]>,
    mine: undefined as unknown as import("vue").Ref<DashboardSummary[]>,
    shared: undefined as unknown as import("vue").Ref<DashboardSummary[]>,
    publicOnly: undefined as unknown as import("vue").Ref<DashboardSummary[]>,
    fetchDashboards: vi.fn(),
    fetchHome: vi.fn(),
    createDashboard: vi.fn(),
  },
  push: vi.fn(),
}));
vi.mock("../composables/dashboardStore", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../composables/dashboardStore")>()),
  useDashboardStore: () => store,
}));
vi.mock("@agdb-studio/auth/src/account", async () => {
  const { ref } = await import("vue");
  return { useAccount: () => ({ username: ref("me") }) };
});
vi.mock("vue-router", () => ({ useRouter: () => ({ push }) }));

import DashboardList from "./DashboardList.vue";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { useAdmin } from "@agdb-studio/router/src/admin";

const { modal, handleConfirm, closeModal } = useModal();
const { setInputValue } = useContentInputs();

const summary = (
  owner: string,
  name: string,
  init: Partial<DashboardSummary> = {},
): DashboardSummary => ({
  owner,
  name,
  updated: 0,
  role: "admin",
  public: false,
  member: true,
  ...init,
});

const mountList = async () => {
  const wrapper = mount(DashboardList, {
    global: {
      stubs: {
        DashboardTable: {
          props: ["title", "dashboards"],
          template:
            '<section class="table"><h3>{{ title }}</h3><span v-for="d of dashboards" class="row">{{ d.owner }}/{{ d.name }}</span><slot v-if="!dashboards.length" name="empty" /></section>',
        },
      },
    },
  });
  await flushPromises();
  return wrapper;
};

const sections = (wrapper: Awaited<ReturnType<typeof mountList>>) =>
  wrapper.findAll(".table").map((t) => ({
    title: t.find("h3").text(),
    rows: t.findAll(".row").map((r) => r.text()),
  }));

const setAdmin = () => {
  const { isAdmin, isAdminView } = useAdmin();
  (isAdmin as Ref<boolean>).value = true;
  (isAdminView as Ref<boolean>).value = true;
};

describe("DashboardList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closeModal();
    store.dashboards = ref([]);
    store.mine = computed(() =>
      store.dashboards.value.filter((d) => d.owner === "me"),
    );
    store.shared = computed(() =>
      store.dashboards.value.filter((d) => d.owner !== "me" && d.member),
    );
    store.publicOnly = computed(() =>
      store.dashboards.value.filter((d) => d.owner !== "me" && !d.member),
    );
    store.fetchDashboards.mockResolvedValue(undefined);
    store.fetchHome.mockResolvedValue(undefined);
  });

  it("shows own, shared and public dashboards in their sections", async () => {
    store.dashboards.value = [
      summary("me", "ops"),
      summary("ann", "sales", { role: "read", public: true }),
      summary("bob", "kpi", { role: "read", public: true, member: false }),
    ];
    const wrapper = await mountList();
    expect(store.fetchHome).toHaveBeenCalled();
    expect(sections(wrapper)).toEqual([
      { title: "My dashboards", rows: ["me/ops"] },
      { title: "Shared with me", rows: ["ann/sales"] },
      { title: "Public", rows: ["bob/kpi"] },
    ]);
  });

  it("hides empty shared and public sections", async () => {
    store.dashboards.value = [summary("me", "ops")];
    const wrapper = await mountList();
    expect(sections(wrapper).map((s) => s.title)).toEqual(["My dashboards"]);
  });

  it("shows an empty state, also when loading fails", async () => {
    store.fetchDashboards.mockRejectedValueOnce(new Error("offline"));
    const wrapper = await mountList();
    expect(wrapper.find('[data-testid="dashboards-empty"]').text()).toContain(
      "pin a query",
    );
  });

  it("creates a dashboard and opens it", async () => {
    store.createDashboard.mockResolvedValueOnce({ owner: "me", name: "new" });
    const wrapper = await mountList();
    await wrapper.find('[data-testid="dashboard-create"]').trigger("click");
    expect(modal.header).toBe("New dashboard");
    expect(modal.content).toHaveLength(1);
    const rule = modal.content[0]!.input!.rules![0]!;
    expect(rule("bad!")).toBeDefined();
    expect(rule(undefined as unknown as string)).toBeDefined();
    setInputValue(KEY_MODAL, "name", " new ");
    await handleConfirm();
    await flushPromises();
    expect(store.createDashboard).toHaveBeenCalledWith("new", undefined);
    expect(push).toHaveBeenCalledWith({
      name: "dashboard",
      params: { owner: "me", name: "new" },
    });
  });

  it("lists all dashboards in one table in the admin view", async () => {
    setAdmin();
    store.dashboards.value = [
      summary("ann", "sales"),
      summary("bob", "kpi", { public: true }),
    ];
    const wrapper = await mountList();
    expect(store.fetchHome).not.toHaveBeenCalled();
    expect(sections(wrapper)).toEqual([
      { title: "All dashboards", rows: ["ann/sales", "bob/kpi"] },
    ]);
  });

  it("shows an empty admin table", async () => {
    setAdmin();
    const wrapper = await mountList();
    expect(wrapper.find('[data-testid="dashboards-empty"]').text()).toBe(
      "No dashboards yet.",
    );
  });

  it("needs an owner to create a dashboard in the admin view", async () => {
    setAdmin();
    const wrapper = await mountList();
    await wrapper.find('[data-testid="dashboard-create"]').trigger("click");
    setInputValue(KEY_MODAL, "owner", "  ");
    setInputValue(KEY_MODAL, "name", "new");
    await handleConfirm();
    await flushPromises();
    expect(store.createDashboard).not.toHaveBeenCalled();
  });

  it("creates a dashboard for a user in the admin view", async () => {
    setAdmin();
    store.createDashboard.mockResolvedValueOnce({ owner: "ann", name: "new" });
    const wrapper = await mountList();
    await wrapper.find('[data-testid="dashboard-create"]').trigger("click");
    expect(modal.content.map((c) => c.input?.key)).toEqual(["owner", "name"]);
    setInputValue(KEY_MODAL, "owner", " ann ");
    setInputValue(KEY_MODAL, "name", "new");
    await handleConfirm();
    await flushPromises();
    expect(store.createDashboard).toHaveBeenCalledWith("new", "ann");
    expect(push).toHaveBeenCalledWith({
      name: "admin-dashboard",
      params: { owner: "ann", name: "new" },
    });
  });
});
