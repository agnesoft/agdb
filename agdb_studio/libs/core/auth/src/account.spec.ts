import {
  vi,
  describe,
  it,
  beforeAll,
  beforeEach,
  afterAll,
  expect,
} from "vitest";
import { ref, nextTick } from "vue";
import { user_status } from "@agdb-studio/testing/mocks/apiMock";

const isLoggedIn = ref(true);
const token = ref<string | undefined>("test");

vi.mock("./auth", () => ({
  useAuth: vi.fn().mockReturnValue({ isLoggedIn, token }),
}));

type AccountModule = typeof import("./account");
let useAccount: AccountModule["useAccount"];

describe("useAccount", () => {
  beforeAll(async () => {
    vi.useFakeTimers();
    vi.resetModules();
    const mod = await import("./account");
    useAccount = mod.useAccount;
  });

  afterAll(() => {
    vi.useRealTimers();
  });

  beforeEach(() => {
    vi.clearAllMocks();
    vi.clearAllTimers();
    isLoggedIn.value = true;
    token.value = "test";
  });

  it("fetches user status", async () => {
    user_status.mockResolvedValueOnce({
      data: { username: "test", admin: true },
    });
    const { username, admin, fetchUserStatus } = useAccount();
    await fetchUserStatus();

    expect(username.value).toBe("test");
    expect(admin.value).toBe(true);
  });

  it("handles token watcher errors gracefully", async () => {
    user_status.mockRejectedValueOnce(new Error("401"));
    token.value = "expired-token";
    await nextTick();
    await vi.advanceTimersByTimeAsync(0);

    expect(user_status).toHaveBeenCalled();
  });

  it("does nothing if not logged in", async () => {
    isLoggedIn.value = false;
    const { username, admin, fetchUserStatus } = useAccount();
    await fetchUserStatus();

    expect(username.value).toBe(undefined);
    expect(admin.value).toBe(false);
  });

  describe("token health check", () => {
    beforeEach(async () => {
      user_status.mockResolvedValue({
        data: { username: "test", admin: false },
      });
      isLoggedIn.value = false;
      await nextTick();
      vi.clearAllTimers();
      isLoggedIn.value = true;
      await nextTick();
      vi.clearAllMocks();
    });

    it("periodically validates token while logged in", async () => {
      user_status.mockResolvedValue({
        data: { username: "test", admin: false },
      });
      await vi.advanceTimersByTimeAsync(60000);
      expect(user_status).toHaveBeenCalled();

      const callCount = user_status.mock.calls.length;
      await vi.advanceTimersByTimeAsync(60000);
      expect(user_status.mock.calls.length).toBeGreaterThan(callCount);
    });

    it("stops periodic validation on logout", async () => {
      user_status.mockResolvedValue({
        data: { username: "test", admin: false },
      });
      isLoggedIn.value = false;
      await nextTick();

      user_status.mockClear();
      await vi.advanceTimersByTimeAsync(60000);
      expect(user_status).not.toHaveBeenCalled();
    });

    it("handles interval errors gracefully", async () => {
      user_status.mockRejectedValue(new Error("500"));
      await vi.advanceTimersByTimeAsync(60000);

      expect(user_status).toHaveBeenCalled();
    });

    it("does not create duplicate intervals", async () => {
      user_status.mockResolvedValue({
        data: { username: "test", admin: false },
      });
      isLoggedIn.value = false;
      await nextTick();
      vi.clearAllTimers();
      isLoggedIn.value = true;
      await nextTick();

      user_status.mockClear();
      await vi.advanceTimersByTimeAsync(60000);
      expect(user_status).toHaveBeenCalledTimes(1);
    });
  });
});
