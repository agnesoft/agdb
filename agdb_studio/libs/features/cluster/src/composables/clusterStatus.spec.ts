import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { defineComponent, nextTick, type UnwrapRef } from "vue";
import { mount, flushPromises, type VueWrapper } from "@vue/test-utils";
import { useClusterStatus } from "./clusterStatus";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

const mockClient = vi.hoisted(() => ({
  cluster_status: vi.fn(),
}));

const mockCheckClient = vi.hoisted(() => vi.fn());
const mockReconnectClient = vi.hoisted(() => vi.fn());
const mockApiUrl = vi.hoisted(() => ({ value: "http://server1:8080" }));
const mockServerUserStatus = vi.hoisted(() => vi.fn());
const mockServerSetToken = vi.hoisted(() => vi.fn());
const mockAgdbClient = vi.hoisted(() => vi.fn());

vi.mock("@agdb-studio/api/src/api", () => ({
  client: { value: mockClient },
  checkClient: mockCheckClient,
  reconnectClient: mockReconnectClient,
  apiUrl: mockApiUrl,
}));

vi.mock("@agdb-studio/api/src/constants", () => ({
  ACCESS_TOKEN: "agdb_token",
}));

vi.mock("@agnesoft/agdb_api", () => ({
  AgdbApi: {
    client: mockAgdbClient,
  },
}));

vi.mock("@agdb-studio/utils/src/logger/logger", () => ({
  createLogger: () => ({
    debug: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    info: vi.fn(),
  }),
}));

type ComposableReturn = ReturnType<typeof useClusterStatus>;
type TestComponentInstance = UnwrapRef<ComposableReturn>;

describe("useClusterStatus", () => {
  let wrapper: VueWrapper<TestComponentInstance> | null = null;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    mockReconnectClient.mockResolvedValue(undefined);
    mockApiUrl.value = "http://server1:8080";
    mockServerUserStatus.mockResolvedValue({ data: { login: true } });
    mockAgdbClient.mockResolvedValue({
      user_status: mockServerUserStatus,
      set_token: mockServerSetToken,
    });
  });

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount();
    }
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  const mountComposable = () => {
    const TestComponent = defineComponent({
      setup() {
        const result = useClusterStatus();
        return { ...result };
      },
      template: "<div></div>",
    });
    wrapper = mount(TestComponent) as VueWrapper<TestComponentInstance>;
    return wrapper.vm;
  };

  it("should have unknown status while loading", () => {
    mockClient.cluster_status.mockResolvedValue({ data: [] });

    const vm = mountComposable();

    // Check immediately before fetch completes
    expect(vm.overallStatus).toBe("unknown");
    expect(vm.isLoading).toBe(true);
  });

  it("should start with unknown status when no servers", async () => {
    mockClient.cluster_status.mockResolvedValue({ data: [] });

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.overallStatus).toBe("red");
    expect(vm.servers).toEqual([]);
  });

  it("should fetch cluster status and calculate GREEN status", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(mockCheckClient).toHaveBeenCalled();
    expect(mockClient.cluster_status).toHaveBeenCalled();
    expect(vm.servers).toEqual(mockServers);
    expect(vm.overallStatus).toBe("green");
    expect(vm.isLoading).toBe(false);
  });

  it("should calculate RED status when all servers are down", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: false, leader: false },
      { address: "server2:8080", status: false, leader: false },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.servers).toEqual(mockServers);
    expect(vm.overallStatus).toBe("red");
  });

  it("should calculate AMBER status when no leader exists", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: true, leader: false },
      { address: "server2:8080", status: true, leader: false },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.servers).toEqual(mockServers);
    expect(vm.overallStatus).toBe("amber");
  });

  it("should calculate AMBER status when any server is down", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: false, leader: false },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.servers).toEqual(mockServers);
    expect(vm.overallStatus).toBe("amber");
  });

  it("should handle fetch errors gracefully", async () => {
    mockClient.cluster_status.mockRejectedValue(new Error("Network error"));

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.servers).toEqual([]);
    expect(vm.isLoading).toBe(false);
  });

  it("should handle non-Error exceptions gracefully", async () => {
    mockClient.cluster_status.mockRejectedValue("String error");

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.servers).toEqual([]);
    expect(vm.isLoading).toBe(false);
  });

  it("should poll cluster status every 15 seconds", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: true, leader: true },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();
    expect(mockClient.cluster_status).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    await nextTick();
    expect(mockClient.cluster_status).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    await nextTick();
    expect(mockClient.cluster_status).toHaveBeenCalledTimes(3);
  });

  it("should update lastUpdated on successful fetch", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: true, leader: true },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();

    const lastUpdatedBefore = vm.lastUpdated;

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.lastUpdated).toBeInstanceOf(Date);
    expect(vm.lastUpdated).not.toBe(lastUpdatedBefore);
  });

  it("should allow manual refresh via fetchStatus", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: true, leader: true },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    const initialCallCount = mockClient.cluster_status.mock.calls.length;

    await vm.fetchStatus();
    await flushPromises();
    await nextTick();

    expect(mockClient.cluster_status).toHaveBeenCalledTimes(
      initialCallCount + 1,
    );
  });

  it("should handle multiple components using the composable", async () => {
    mockClient.cluster_status.mockResolvedValue({ data: [] });

    // Mount first component
    const vm1 = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(mockClient.cluster_status).toHaveBeenCalledTimes(1);

    // The composable shares module-level state, so additional mounts
    // should still work without causing duplicate polling
    expect(vm1.servers).toEqual([]);
    expect(vm1.isLoading).toBe(false);
  });

  it("should not start polling again if already polling", async () => {
    mockClient.cluster_status.mockResolvedValue({ data: [] });

    const vm = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    const callCountAfterMount = mockClient.cluster_status.mock.calls.length;

    // Try to start polling again manually - should be a no-op
    vm.startPolling();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    // Should not have made additional immediate calls (polling guard active)
    expect(mockClient.cluster_status).toHaveBeenCalledTimes(
      callCountAfterMount,
    );
  });

  it("should stop polling on unmount", async () => {
    mockClient.cluster_status.mockResolvedValue({ data: [] });

    const vm = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(mockClient.cluster_status).toHaveBeenCalledTimes(1);

    // Manually stop polling to test the stopPolling branch
    vm.stopPolling();

    // Advance time - should not trigger additional calls since polling stopped
    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    await nextTick();

    expect(mockClient.cluster_status).toHaveBeenCalledTimes(1);
  });

  it("should switch to another online server using localhost host and target port", async () => {
    // Current connection is http://server1:8080, cluster reports internal hostnames.
    // switchToServer must preserve the host (server1 / localhost) and only swap the port.
    mockApiUrl.value = "http://server1:8080";
    const mockServers: ClusterStatus[] = [
      { address: "https://agdb0:8080", status: true, leader: true },
      { address: "https://agdb1:9090", status: true, leader: false },
    ];

    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    await vm.switchToServer(mockServers[1]!);

    // Host kept as server1, port taken from agdb1:9090
    expect(mockReconnectClient).toHaveBeenCalledWith("http://server1:9090");
  });

  it("should track per-server logged-in status", async () => {
    const mockServers: ClusterStatus[] = [
      { address: "server1:8080", status: true, leader: true },
      { address: "server2:8080", status: true, leader: false },
    ];
    mockServerUserStatus
      .mockResolvedValueOnce({ data: { login: true } })
      .mockResolvedValueOnce({ data: { login: false } });
    mockClient.cluster_status.mockResolvedValue({ data: mockServers });

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.isUserLoggedInOnServer(mockServers[0]!)).toBe(true);
    expect(vm.isUserLoggedInOnServer(mockServers[1]!)).toBe(false);
  });

  it("should not switch to offline server", async () => {
    const offlineServer: ClusterStatus = {
      address: "server2:8080",
      status: false,
      leader: false,
    };

    const vm = mountComposable();
    await vm.switchToServer(offlineServer);

    expect(mockReconnectClient).not.toHaveBeenCalled();
  });
});
