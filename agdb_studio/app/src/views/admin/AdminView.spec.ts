import { vi, describe, it, beforeEach, expect } from "vitest";
import AdminView from "./AdminView.vue";
import { shallowMount, flushPromises } from "@vue/test-utils";
import type { AdminStatus, ClusterStatus } from "@agnesoft/agdb_api/openapi";
import { ref } from "vue";

const mockAdminStatus: AdminStatus = {
  dbs: 5,
  log_level: "info",
  logged_in_users: 3,
  memory: 268435456,
  size: 1073741824,
  uptime: 90061,
  users: 10,
};

const mockServers: ClusterStatus[] = [
  {
    address: "server1:8080",
    status: true,
    leader: true,
    executed_index: 42,
    resync: false,
  },
  {
    address: "server2:8080",
    status: true,
    leader: false,
    executed_index: 42,
    resync: false,
  },
  {
    address: "server3:8080",
    status: false,
    leader: false,
    executed_index: 38,
    resync: true,
  },
];

const mockFetchAdminStatus = vi.fn();
const mockFetchClusterStatus = vi.fn();

const mockAdminStatusRef = ref<AdminStatus | null>(mockAdminStatus);
const mockAdminLoadingRef = ref(false);
const mockServersRef = ref<ClusterStatus[]>(mockServers);
const mockClusterLoadingRef = ref(false);

vi.mock("@/composables/adminStatus", () => ({
  useAdminStatus: () => ({
    status: mockAdminStatusRef,
    isLoading: mockAdminLoadingRef,
    lastUpdated: ref(new Date()),
    fetchStatus: mockFetchAdminStatus,
    startPolling: vi.fn(),
    stopPolling: vi.fn(),
  }),
}));

vi.mock("@agdb-studio/cluster/src/composables/clusterStatus", () => ({
  useClusterStatus: () => ({
    servers: mockServersRef,
    overallStatus: ref("green"),
    isLoading: mockClusterLoadingRef,
    lastUpdated: ref(new Date()),
    switchingServerAddress: ref(null),
    loggedInByServerAddress: ref({}),
    activeServer: ref(undefined),
    activeNodeLabel: ref(":8080"),
    isServerActive: vi.fn(),
    isUserLoggedInOnServer: vi.fn(),
    switchToServer: vi.fn(),
    fetchStatus: mockFetchClusterStatus,
    startPolling: vi.fn(),
    stopPolling: vi.fn(),
  }),
}));

vi.mock("@agdb-studio/utils/src/format/format", () => ({
  formatBytes: (bytes: number) => `${bytes} bytes`,
  formatUptime: (seconds: number) => `${seconds}s`,
}));

describe("AdminView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAdminStatusRef.value = mockAdminStatus;
    mockAdminLoadingRef.value = false;
    mockServersRef.value = mockServers;
    mockClusterLoadingRef.value = false;
  });

  it("renders server status fields", () => {
    const wrapper = shallowMount(AdminView);

    expect(wrapper.find("[data-testid='stat-dbs']").text()).toBe("5");
    expect(wrapper.find("[data-testid='stat-users']").text()).toBe("10");
    expect(wrapper.find("[data-testid='stat-logged-in']").text()).toBe("3");
    expect(wrapper.find("[data-testid='stat-log-level']").text()).toBe("info");
    expect(wrapper.find("[data-testid='stat-memory']").text()).toBe(
      "268435456 bytes",
    );
    expect(wrapper.find("[data-testid='stat-size']").text()).toBe(
      "1073741824 bytes",
    );
    expect(wrapper.find("[data-testid='stat-uptime']").text()).toBe("90061s");
  });

  it("renders cluster node table", () => {
    const wrapper = shallowMount(AdminView);

    const rows = wrapper.findAll("tbody tr");
    expect(rows).toHaveLength(3);
    expect(rows[0]!.text()).toContain("server1:8080");
    expect(rows[0]!.text()).toContain("Connected");
    expect(rows[1]!.text()).toContain("server2:8080");
    expect(rows[2]!.text()).toContain("Disconnected");
    expect(rows[2]!.text()).toContain("Resyncing");
  });

  it("shows spinner while admin status is loading", () => {
    mockAdminLoadingRef.value = true;

    const wrapper = shallowMount(AdminView);

    expect(wrapper.find("[data-testid='server-status']").exists()).toBe(false);
    expect(wrapper.findComponent({ name: "SpinnerIcon" }).exists()).toBe(true);
  });

  it("shows error text when admin status is null", () => {
    mockAdminStatusRef.value = null;

    const wrapper = shallowMount(AdminView);

    expect(wrapper.find("[data-testid='admin-status-error']").exists()).toBe(
      true,
    );
  });

  it("shows empty text when no cluster nodes", () => {
    mockServersRef.value = [];

    const wrapper = shallowMount(AdminView);

    expect(wrapper.find("[data-testid='no-cluster']").exists()).toBe(true);
  });

  it("calls both fetch functions on refresh", async () => {
    mockFetchAdminStatus.mockResolvedValue(undefined);
    mockFetchClusterStatus.mockResolvedValue(undefined);

    const wrapper = shallowMount(AdminView);
    await wrapper.find("[data-testid='refresh-button']").trigger("click");

    expect(mockFetchAdminStatus).toHaveBeenCalled();
    expect(mockFetchClusterStatus).toHaveBeenCalled();
  });

  it("disables refresh button while refreshing", async () => {
    let resolveAdmin!: () => void;
    mockFetchAdminStatus.mockReturnValue(
      new Promise<void>((r) => {
        resolveAdmin = r;
      }),
    );
    mockFetchClusterStatus.mockResolvedValue(undefined);

    const wrapper = shallowMount(AdminView);
    const btn = wrapper.find("[data-testid='refresh-button']");

    btn.trigger("click");
    await wrapper.vm.$nextTick();
    expect((btn.element as HTMLButtonElement).disabled).toBe(true);

    resolveAdmin();
    await flushPromises();
    expect((btn.element as HTMLButtonElement).disabled).toBe(false);
  });

  it("completes refresh even if one fetch rejects", async () => {
    mockFetchAdminStatus.mockRejectedValue(new Error("fail"));
    mockFetchClusterStatus.mockResolvedValue(undefined);

    const wrapper = shallowMount(AdminView);
    const btn = wrapper.find("[data-testid='refresh-button']");

    await btn.trigger("click");
    await wrapper.vm.$nextTick();

    expect(mockFetchClusterStatus).toHaveBeenCalled();
    expect((btn.element as HTMLButtonElement).disabled).toBe(false);
  });
});
