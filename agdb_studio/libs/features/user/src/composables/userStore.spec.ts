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
      username: "admin_user",
      admin: true,
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
    expect(users.value[0]?.username).toBe("admin_user");
    expect(users.value[0]?.admin).toBe(true);
  });

  it("adds user", async () => {
    await addUser({ username: "test_user", password: "test_password" });
    expect(admin_user_add).toHaveBeenCalledOnce();
  });

  it("sorts users alphabetically when admin flags are equal", async () => {
    admin_user_list.mockResolvedValueOnce({
      data: [
        {
          username: "zeta",
          admin: false,
          login: false,
        },
        {
          username: "alpha",
          admin: false,
          login: true,
        },
      ],
    });

    await fetchUsers();

    expect(users.value.map((user) => user.username)).toEqual(["alpha", "zeta"]);
  });
});
