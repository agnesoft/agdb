import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { defineComponent, nextTick, type UnwrapRef } from "vue";
import { mount, flushPromises, type VueWrapper } from "@vue/test-utils";
import { useAdminStatus, resetAdminStatusState } from "./adminStatus";
import type { AdminStatus } from "@agnesoft/agdb_api/openapi";

const mockClient = vi.hoisted(() => ({
  admin_status: vi.fn(),
  set_log_level: vi.fn(),
}));

const mockCheckClient = vi.hoisted(() => vi.fn());

const mockIsLoggedIn = vi.hoisted(() => ({ value: true }));

vi.mock("@agdb-studio/api/src/api", () => ({
  client: { value: mockClient },
  checkClient: mockCheckClient,
}));

vi.mock("@agdb-studio/auth/src/auth", () => ({
  useAuth: () => ({
    isLoggedIn: mockIsLoggedIn,
  }),
}));

const mockAddNotification = vi.hoisted(() => vi.fn());

vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification: mockAddNotification,
}));

vi.mock("@agdb-studio/utils/src/logger/logger", () => ({
  createLogger: () => ({
    debug: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    info: vi.fn(),
  }),
}));

const mockAdminStatus: AdminStatus = {
  dbs: 5,
  log_level: "info",
  logged_in_users: 3,
  memory: 268435456,
  size: 1073741824,
  uptime: 86400,
  users: 10,
};

type ComposableReturn = ReturnType<typeof useAdminStatus>;
type TestComponentInstance = UnwrapRef<ComposableReturn>;

describe("useAdminStatus", () => {
  let wrapper: VueWrapper<TestComponentInstance> | null = null;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    resetAdminStatusState();
    mockClient.admin_status.mockResolvedValue({ data: mockAdminStatus });
  });

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount();
    }
    vi.useRealTimers();
  });

  const mountComposable = () => {
    const TestComponent = defineComponent({
      setup() {
        const result = useAdminStatus();
        return { ...result };
      },
      template: "<div></div>",
    });
    wrapper = mount(TestComponent) as VueWrapper<TestComponentInstance>;
    return wrapper.vm;
  };

  it("should have unknown status while loading", () => {
    const vm = mountComposable();
    expect(vm.isLoading).toBe(true);
    expect(vm.status).toBeNull();
  });

  it("should fetch admin status on mount", async () => {
    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(mockCheckClient).toHaveBeenCalled();
    expect(mockClient.admin_status).toHaveBeenCalled();
    expect(vm.status).toEqual(mockAdminStatus);
    expect(vm.isLoading).toBe(false);
    expect(vm.lastUpdated).toBeInstanceOf(Date);
  });

  it("should handle fetch errors gracefully", async () => {
    mockClient.admin_status.mockRejectedValue(new Error("Network error"));

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.status).toBeNull();
    expect(vm.isLoading).toBe(false);
  });

  it("should handle non-Error exceptions gracefully", async () => {
    mockClient.admin_status.mockRejectedValue("String error");

    const vm = mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.status).toBeNull();
    expect(vm.isLoading).toBe(false);
  });

  it("should poll admin status every 15 seconds", async () => {
    mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();
    expect(mockClient.admin_status).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    await nextTick();
    expect(mockClient.admin_status).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    await nextTick();
    expect(mockClient.admin_status).toHaveBeenCalledTimes(3);
  });

  it("should allow manual refresh via fetchStatus", async () => {
    const vm = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    const initialCallCount = mockClient.admin_status.mock.calls.length;

    await vm.fetchStatus();
    await flushPromises();
    await nextTick();

    expect(mockClient.admin_status).toHaveBeenCalledTimes(initialCallCount + 1);
  });

  it("should not start polling again if already polling", async () => {
    const vm = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    const callCountAfterMount = mockClient.admin_status.mock.calls.length;

    vm.startPolling();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(mockClient.admin_status).toHaveBeenCalledTimes(callCountAfterMount);
  });

  it("should stop polling on unmount", async () => {
    const vm = mountComposable();

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(mockClient.admin_status).toHaveBeenCalledTimes(1);

    vm.stopPolling();

    await vi.advanceTimersByTimeAsync(15000);
    await flushPromises();
    await nextTick();

    expect(mockClient.admin_status).toHaveBeenCalledTimes(1);
  });

  it("should skip fetch when not logged in", async () => {
    mockIsLoggedIn.value = false;
    mountComposable();
    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(mockClient.admin_status).not.toHaveBeenCalled();
    mockIsLoggedIn.value = true;
  });

  it("should update lastUpdated on successful fetch", async () => {
    const vm = mountComposable();

    const lastUpdatedBefore = vm.lastUpdated;

    await vi.advanceTimersByTimeAsync(0);
    await flushPromises();
    await nextTick();

    expect(vm.lastUpdated).toBeInstanceOf(Date);
    expect(vm.lastUpdated).not.toBe(lastUpdatedBefore);
  });

  describe("setLogLevel", () => {
    it("should call set_log_level and re-fetch status", async () => {
      mockClient.set_log_level.mockResolvedValue({});

      const vm = mountComposable();
      await vi.advanceTimersByTimeAsync(0);
      await flushPromises();
      await nextTick();

      const callsBefore = mockClient.admin_status.mock.calls.length;

      await vm.setLogLevel("debug");
      await flushPromises();
      await nextTick();

      expect(mockClient.set_log_level).toHaveBeenCalledWith({
        new_level: "debug",
      });
      expect(mockClient.admin_status).toHaveBeenCalledTimes(callsBefore + 1);
    });

    it("should show success notification after setting log level", async () => {
      mockClient.set_log_level.mockResolvedValue({});

      const vm = mountComposable();
      await vi.advanceTimersByTimeAsync(0);
      await flushPromises();
      await nextTick();

      await vm.setLogLevel("warn");
      await flushPromises();
      await nextTick();

      expect(mockAddNotification).toHaveBeenCalledWith({
        type: "success",
        title: "Log level updated",
        message: 'Log level set to "warn".',
      });
    });

    it("should set isSettingLogLevel while request is in-flight", async () => {
      let resolveSetLogLevel!: () => void;
      mockClient.set_log_level.mockReturnValue(
        new Promise<void>((r) => {
          resolveSetLogLevel = r;
        }),
      );

      const vm = mountComposable();
      await vi.advanceTimersByTimeAsync(0);
      await flushPromises();
      await nextTick();

      expect(vm.isSettingLogLevel).toBe(false);

      const promise = vm.setLogLevel("trace");
      await nextTick();
      expect(vm.isSettingLogLevel).toBe(true);

      resolveSetLogLevel();
      await promise;
      await flushPromises();
      await nextTick();

      expect(vm.isSettingLogLevel).toBe(false);
    });

    it("should handle set_log_level errors gracefully and re-fetch status", async () => {
      mockClient.set_log_level.mockRejectedValue(new Error("Forbidden"));

      const vm = mountComposable();
      await vi.advanceTimersByTimeAsync(0);
      await flushPromises();
      await nextTick();

      const callsBefore = mockClient.admin_status.mock.calls.length;

      await vm.setLogLevel("error");
      await flushPromises();
      await nextTick();

      expect(vm.isSettingLogLevel).toBe(false);
      expect(mockAddNotification).not.toHaveBeenCalled();
      expect(mockClient.admin_status).toHaveBeenCalledTimes(callsBefore + 1);
    });
  });
});
