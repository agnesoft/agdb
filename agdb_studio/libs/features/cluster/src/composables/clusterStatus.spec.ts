import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { defineComponent, nextTick } from "vue";
import { mount, flushPromises } from "@vue/test-utils";
import { useClusterStatus } from "./clusterStatus";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

const mockClient = vi.hoisted(() => ({
  cluster_status: vi.fn(),
}));

const mockCheckClient = vi.hoisted(() => vi.fn());

vi.mock("@agdb-studio/api/src/api", () => ({
  client: { value: mockClient },
  checkClient: mockCheckClient,
}));

vi.mock("@agdb-studio/utils/src/logger/logger", () => ({
  createLogger: () => ({
    debug: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    info: vi.fn(),
  }),
}));

describe("useClusterStatus", () => {
  let wrapper: any;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
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
    wrapper = mount(TestComponent);
    return wrapper.vm;
  };

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
});
