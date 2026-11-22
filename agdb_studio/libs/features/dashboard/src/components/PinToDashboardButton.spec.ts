import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { ref } from "vue";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});

const { store, addNotification } = vi.hoisted(() => ({
  store: {
    dashboards: undefined as unknown as import("vue").Ref<
      { owner: string; name: string; role: string }[]
    >,
    fetchDashboards: vi.fn(),
    createDashboard: vi.fn(),
    appendWidgets: vi.fn(),
  },
  addNotification: vi.fn(),
}));
vi.mock("../composables/dashboardStore", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../composables/dashboardStore")>()),
  useDashboardStore: () => store,
}));
vi.mock("@agdb-studio/auth/src/account", async () => {
  const { ref } = await import("vue");
  return { useAccount: () => ({ username: ref("me") }) };
});
vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification,
}));

import PinToDashboardButton from "./PinToDashboardButton.vue";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import { emptyColorConfig } from "@agdb-studio/query/src/utils/graphColorRules";
import { toQueryRows } from "../utils/widgetRows";
import { NewerDashboardError } from "../utils/dashboardDocument";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";

const { modal, handleConfirm, closeModal, modalIsVisible } = useModal();
const { setInputValue } = useContentInputs();
const set = (key: string, value: string) =>
  setInputValue(KEY_MODAL, key, value);

const Q1 = 'select().ids("users")';
const Q2 = 'search().from("users")';
const queryStore = useQueryStore();

const seed = (exec: string[]) => {
  queryStore.clearQueries();
  queryStore.addQuery({ id: "page" });
  const query = queryStore.getQuery("page")!.value;
  query.rows.exec = toQueryRows(
    exec.map((q, i) => ({ id: `r${i}`, query: q })),
  );
  query.rows.context = toQueryRows([{ id: "c0", query: Q2 }]);
  query.colors = { ...emptyColorConfig(), rows: { r0: 1 } };
  query.view = "table";
};

const mountButton = (queryId?: string) =>
  mount(PinToDashboardButton, {
    props: { owner: "o", db: "d", queryId: queryId ?? "page" },
  });

const open = async (wrapper: ReturnType<typeof mountButton>) => {
  await wrapper.find("button").trigger("click");
  await flushPromises();
};

describe("PinToDashboardButton", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closeModal();
    seed([Q1]);
    store.dashboards = ref([{ owner: "me", name: "ops", role: "admin" }]);
    store.fetchDashboards.mockResolvedValue(undefined);
  });

  it.each([
    ["without a query entry", undefined],
    ["for an unknown entry", "missing"],
  ])("is disabled %s", (_, queryId) => {
    const wrapper = mount(PinToDashboardButton, {
      props: { owner: "o", db: "d", queryId },
    });
    expect(wrapper.find("button").attributes("disabled")).toBeDefined();
    expect(wrapper.find("button").attributes("title")).toContain("Write");
  });

  it("is disabled while a row cannot run yet", () => {
    seed([Q1, "select()"]);
    const button = mountButton().find("button");
    expect(button.attributes("disabled")).toBeDefined();
    expect(button.attributes("title")).toContain("Finish");
  });

  it("is disabled while the exec tab is empty", () => {
    seed([]);
    expect(mountButton().find("button").attributes("disabled")).toBeDefined();
  });

  it("adds the whole query as one widget to an existing dashboard", async () => {
    const wrapper = mountButton();
    expect(wrapper.find("button").attributes("title")).toContain("Add");
    await open(wrapper);
    expect(modal.header).toBe("Add to dashboard");
    set("title", "Users");
    await handleConfirm();
    await flushPromises();
    expect(store.createDashboard).not.toHaveBeenCalled();
    expect(store.appendWidgets).toHaveBeenCalledWith(
      { owner: "me", name: "ops" },
      [
        expect.objectContaining({
          owner: "o",
          db: "d",
          title: "Users",
          view: "table",
          rows: {
            exec: [{ id: "r0", query: Q1 }],
            context: [{ id: "c0", query: Q2 }],
          },
          colors: expect.objectContaining({ rows: { r0: 1 } }),
        }),
      ],
    );
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        message: '"Users" added to ops.',
      }),
    );
  });

  it("creates a new dashboard", async () => {
    store.dashboards.value = [];
    seed([Q1, Q2]);
    await open(mountButton());
    const rule = modal.content[1]!.input!.rules![0]!;
    expect(rule("bad!")).toBeDefined();
    expect(rule(undefined as unknown as string)).toBeDefined();
    set("new_name", "fresh");
    store.createDashboard.mockResolvedValueOnce({ owner: "me", name: "fresh" });
    await handleConfirm();
    await flushPromises();
    expect(store.createDashboard).toHaveBeenCalledWith("fresh");
    const [[key, widgets]] = store.appendWidgets.mock.calls;
    expect(key).toEqual({ owner: "me", name: "fresh" });
    expect(widgets).toHaveLength(1);
    expect(widgets[0].title).toBe("o/d");
    expect(widgets[0].rows.exec).toHaveLength(2);
  });

  it("does not create the new dashboard again when retrying a failed pin", async () => {
    store.dashboards.value = [];
    store.createDashboard.mockResolvedValueOnce({ owner: "me", name: "fresh" });
    store.appendWidgets.mockRejectedValueOnce(new Error("offline"));
    await open(mountButton());
    set("new_name", "fresh");
    await handleConfirm();
    await flushPromises();
    expect(modalIsVisible.value).toBe(true);
    await handleConfirm();
    await flushPromises();
    expect(store.createDashboard).toHaveBeenCalledTimes(1);
    expect(store.appendWidgets).toHaveBeenCalledTimes(2);
  });

  it("skips the new name rule for existing dashboards", async () => {
    await open(mountButton());
    expect(modal.content[1]!.input!.rules![0]!("bad!")).toBeUndefined();
    useContentInputs().getContentInputs(KEY_MODAL)?.delete("title");
    await handleConfirm();
    await flushPromises();
    expect(store.appendWidgets).toHaveBeenCalled();
  });

  it("offers only dashboards the user can edit, labelled owner/name", async () => {
    store.dashboards.value = [
      { owner: "ann", name: "seen", role: "read" },
      { owner: "ann", name: "ops", role: "write" },
    ];
    await open(mountButton());
    const input = modal.content[0]!.input!;
    expect(input.options).toEqual([
      { value: "ann/ops", label: "ann/ops" },
      { value: ".", label: "+ New dashboard" },
    ]);
    expect(input.value).toBe("ann/ops");
    await handleConfirm();
    await flushPromises();
    expect(store.appendWidgets).toHaveBeenCalledWith(
      { owner: "ann", name: "ops" },
      expect.any(Array),
    );
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({ message: '"o/d" added to ann/ops.' }),
    );
  });

  it("explains a dashboard it cannot add to and keeps the dialog open", async () => {
    store.appendWidgets.mockRejectedValueOnce(new NewerDashboardError(3));
    await open(mountButton());
    await handleConfirm();
    await flushPromises();
    expect(modalIsVisible.value).toBe(true);
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: "error", title: "Not added" }),
    );
  });

  it("does nothing when the dashboards cannot be listed", async () => {
    store.fetchDashboards.mockRejectedValueOnce(new Error("offline"));
    await open(mountButton());
    expect(modalIsVisible.value).toBe(false);
  });
});
