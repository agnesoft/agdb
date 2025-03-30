import { vi, describe, it, beforeEach, expect } from "vitest";
import { useUserStore } from "./userStore";
import {
  admin_user_add,
  admin_user_list,
} from "@agdb-studio/testing/mocks/apiMock";

admin_user_list.mockResolvedValue({
  data: [
    {
      username: "test_user",
      admin: false,
      login: false,
    },
    {
      username: "test_user2",
      admin: false,
      login: false,
    },
  ],
});

const { users, fetchUsers, addUser } = useUserStore();

describe("userStore.ts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches users", async () => {
    expect(users.value.length).toBe(0);
    await fetchUsers();
    expect(admin_user_list).toHaveBeenCalledOnce();
    expect(users.value.length).toBe(2);
  });

  it("adds user", async () => {
    await addUser({ username: "test_user", password: "test_password" });
    expect(admin_user_add).toHaveBeenCalledOnce();
  });
});
