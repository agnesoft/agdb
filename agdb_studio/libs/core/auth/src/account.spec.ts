import { useAccount } from "./account";
import { user_status } from "@agdb-studio/testing/mocks/apiMock";
import { vi, describe, it, beforeEach, expect } from "vitest";

const { isLoggedIn, token } = vi.hoisted(() => {
  return {
    isLoggedIn: { value: true },
    token: { value: "test" },
  };
});
vi.mock("./auth", () => {
  return {
    useAuth: vi.fn().mockReturnValue({
      isLoggedIn,
      token,
    }),
  };
});
describe("useAccount", () => {
  beforeEach(() => {
    vi.clearAllMocks();
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

  it("does nothing if not logged in", async () => {
    isLoggedIn.value = false;
    const { username, admin, fetchUserStatus } = useAccount();
    await fetchUserStatus();

    expect(username.value).toBe(undefined);
    expect(admin.value).toBe(false);
  });
});
