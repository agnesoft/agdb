import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { ref, type Ref } from "vue";
import type { DashboardSummary } from "@agnesoft/agdb_api";

const { store } = vi.hoisted(() => ({
  store: {
    home: undefined as unknown as import("vue").Ref<unknown>,
    setHome: vi.fn(),
    renameDashboard: vi.fn(),
    removeDashboard: vi.fn(),
    leaveDashboard: vi.fn(),
    setPublic: vi.fn(),
  },
}));
vi.mock("../composables/dashboardStore", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../composables/dashboardStore")>()),
  useDashboardStore: () => store,
}));
vi.mock("@agdb-studio/auth/src/account", async () => {
  const { ref } = await import("vue");
  return { useAccount: () => ({ username: ref("me") }) };
});

import DashboardTable from "./DashboardTable.vue";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { useAdmin } from "@agdb-studio/router/src/admin";

const { modal, handleConfirm, closeModal } = useModal();
const { setInputValue } = useContentInputs();

const ops = { owner: "me", name: "ops" };
const shared = { owner: "ann", name: "sales" };
const open = { owner: "bob", name: "kpi" };

const summary = (
  key: { owner: string; name: string },
  init: Partial<DashboardSummary> = {},
): DashboardSummary => ({
  ...key,
  updated: 0,
  role: "admin",
  public: false,
  member: true,
  ...init,
});

const defaults = () => [
  summary(ops, { updated: 1_700_000_000 }),
  summary(shared, { role: "read" }),
];

const mountTable = async (dashboards = defaults()) => {
  const wrapper = mount(DashboardTable, {
    props: { title: "Section", dashboards },
    slots: { empty: '<p class="none">none</p>' },
    global: {
      stubs: {
        RouterLink: {
          props: ["to"],
          template: '<a class="link" :data-route="to.name"><slot /></a>',
        },
        DashboardUsers: {
          props: ["owner", "name", "role"],
          template:
            '<div class="users">{{ owner }}/{{ name }}:{{ role }}</div>',
        },
      },
    },
  });
  await flushPromises();
  return wrapper;
};

type Wrapper = Awaited<ReturnType<typeof mountTable>>;
const rows = (wrapper: Wrapper) => wrapper.findAll(".entry");
const has = (row: ReturnType<Wrapper["find"]>, id: string) =>
  row.find(`[data-testid="${id}"]`).exists();

const setAdmin = () => {
  const { isAdmin, isAdminView } = useAdmin();
  (isAdmin as Ref<boolean>).value = true;
  (isAdminView as Ref<boolean>).value = true;
};

describe("DashboardTable", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closeModal();
    store.home = ref(undefined);
    store.setHome.mockImplementation(async (key) => {
      store.home.value = key;
    });
  });

  it("lists dashboards with links, roles and dates", async () => {
    const wrapper = await mountTable();
    expect(wrapper.find("h3").text()).toBe("Section");
    const links = wrapper.findAll(".link");
    expect(links.map((l) => l.text())).toEqual(["ops", "ann/sales"]);
    expect(links[0]!.attributes("data-route")).toBe("dashboard");
    const [own, other] = rows(wrapper);
    expect(has(own!, "dashboard-role")).toBe(false);
    expect(other!.find('[data-testid="dashboard-role"]').text()).toBe(
      "Read only",
    );
    expect(wrapper.findAll(".updated")[0]!.text()).not.toBe("");
    expect(wrapper.findAll(".updated")[1]!.text()).toBe("");
  });

  it("shows the empty slot without dashboards", async () => {
    const wrapper = await mountTable([]);
    expect(wrapper.find(".none").exists()).toBe(true);
    expect(wrapper.find(".entry").exists()).toBe(false);
  });

  it("renames and deletes own dashboards only", async () => {
    const wrapper = await mountTable();
    const [own, other] = rows(wrapper);
    expect(has(other!, "dashboard-rename")).toBe(false);
    expect(has(other!, "dashboard-remove")).toBe(false);
    expect(has(own!, "dashboard-leave")).toBe(false);

    await own!.find('[data-testid="dashboard-rename"]').trigger("click");
    expect(modal.content[0]!.input!.value).toBe("ops");
    setInputValue(KEY_MODAL, "name", "sales");
    await handleConfirm();
    await flushPromises();
    expect(store.renameDashboard).toHaveBeenCalledWith(ops, "sales");

    await own!.find('[data-testid="dashboard-remove"]').trigger("click");
    expect(modal.header).toBe("Delete dashboard");
    await handleConfirm();
    await flushPromises();
    expect(store.removeDashboard).toHaveBeenCalledWith(ops);
  });

  it("treats a missing name input as empty", async () => {
    const wrapper = await mountTable();
    await wrapper.find('[data-testid="dashboard-rename"]').trigger("click");
    useContentInputs().getContentInputs(KEY_MODAL)?.delete("name");
    await handleConfirm();
    await flushPromises();
    expect(store.renameDashboard).toHaveBeenCalledWith(ops, "");
  });

  it("leaves a shared dashboard", async () => {
    const wrapper = await mountTable();
    await rows(wrapper)[1]!
      .find('[data-testid="dashboard-leave"]')
      .trigger("click");
    expect(modal.header).toBe("Leave dashboard");
    await handleConfirm();
    await flushPromises();
    expect(store.leaveDashboard).toHaveBeenCalledWith(shared);
  });

  it("toggles the home dashboard", async () => {
    store.home.value = { ...ops };
    const wrapper = await mountTable();
    const stars = () => wrapper.findAll('[data-testid="dashboard-home"]');
    expect(stars().map((s) => s.attributes("aria-pressed"))).toEqual([
      "true",
      "false",
    ]);

    await stars()[1]!.trigger("click");
    await flushPromises();
    expect(store.setHome).toHaveBeenLastCalledWith(shared);
    expect(stars().map((s) => s.attributes("aria-pressed"))).toEqual([
      "false",
      "true",
    ]);

    await stars()[1]!.trigger("click");
    await flushPromises();
    expect(store.setHome).toHaveBeenLastCalledWith(undefined);

    store.setHome.mockRejectedValueOnce(new Error("offline"));
    await stars()[0]!.trigger("click");
    await flushPromises();
    expect(stars()[0]!.attributes("aria-pressed")).toBe("false");
  });

  it("shows the users of one dashboard at a time", async () => {
    const wrapper = await mountTable();
    const toggles = () =>
      wrapper.findAll('[data-testid="dashboard-users-toggle"]');
    await toggles()[1]!.trigger("click");
    expect(wrapper.findAll(".users").map((u) => u.text())).toEqual([
      "ann/sales:read",
    ]);
    await toggles()[0]!.trigger("click");
    expect(wrapper.findAll(".users").map((u) => u.text())).toEqual([
      "me/ops:admin",
    ]);
    await toggles()[0]!.trigger("click");
    expect(wrapper.find(".users").exists()).toBe(false);
  });

  it("makes own dashboards public and private after confirming", async () => {
    const wrapper = await mountTable([
      summary(ops),
      summary({ owner: "me", name: "kpi" }, { public: true }),
      summary(shared, { role: "read" }),
    ]);
    const [priv, pub, other] = rows(wrapper);
    expect(has(other!, "dashboard-public")).toBe(false);
    const toggle = (row: typeof priv) =>
      row!.find('[data-testid="dashboard-public"]');
    expect(toggle(priv).attributes("aria-pressed")).toBe("false");
    expect(toggle(pub).attributes("aria-pressed")).toBe("true");
    // Own dashboards show the toggle, not a badge.
    expect(has(pub!, "dashboard-public-badge")).toBe(false);

    await toggle(priv).trigger("click");
    expect(modal.header).toBe("Make dashboard public");
    await handleConfirm();
    await flushPromises();
    expect(store.setPublic).toHaveBeenLastCalledWith(ops, true);

    await toggle(pub).trigger("click");
    expect(modal.header).toBe("Make dashboard private");
    await handleConfirm();
    await flushPromises();
    expect(store.setPublic).toHaveBeenLastCalledWith(
      { owner: "me", name: "kpi" },
      false,
    );
  });

  it("shows public dashboards of others with a badge, without user actions", async () => {
    const wrapper = await mountTable([
      summary(shared, { role: "write", public: true }),
      summary(open, { role: "read", public: true, member: false }),
    ]);
    const [member, viewer] = rows(wrapper);
    expect(has(member!, "dashboard-public-badge")).toBe(true);
    expect(has(member!, "dashboard-role")).toBe(true);
    expect(has(member!, "dashboard-leave")).toBe(true);

    expect(viewer!.find('[data-testid="dashboard-public-badge"]').text()).toBe(
      "Public",
    );
    expect(has(viewer!, "dashboard-role")).toBe(false);
    expect(has(viewer!, "dashboard-users-toggle")).toBe(false);
    expect(has(viewer!, "dashboard-leave")).toBe(false);
    expect(has(viewer!, "dashboard-home")).toBe(true);
  });

  it("manages every dashboard as the owner in the admin view", async () => {
    setAdmin();
    const wrapper = await mountTable([
      summary(shared, { public: true }),
      summary(open),
    ]);
    const links = wrapper.findAll(".link");
    expect(links.map((l) => l.text())).toEqual(["ann/sales", "bob/kpi"]);
    expect(links[0]!.attributes("data-route")).toBe("admin-dashboard");
    for (const row of rows(wrapper)) {
      expect(has(row, "dashboard-home")).toBe(false);
      expect(has(row, "dashboard-leave")).toBe(false);
      expect(has(row, "dashboard-public-badge")).toBe(false);
      expect(has(row, "dashboard-public")).toBe(true);
      expect(has(row, "dashboard-rename")).toBe(true);
      expect(has(row, "dashboard-remove")).toBe(true);
      expect(has(row, "dashboard-users-toggle")).toBe(true);
    }
    await rows(wrapper)[0]!
      .find('[data-testid="dashboard-remove"]')
      .trigger("click");
    expect(modal.content[0]!.paragraph![1]!.text).toBe("ann/sales");
  });
});
