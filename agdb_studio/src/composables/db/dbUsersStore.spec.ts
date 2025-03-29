import { describe, beforeEach, vi, it, expect } from "vitest";
import { useDbUsersStore } from "./dbUsersStore";
import { db_user_list, db_user_add, db_user_remove } from "@/tests/apiMock";

const dbIdentification = {
  db: "testDb",
  owner: "testOwner",
};
const {
  getDbUsers,
  fetchDbUsers,
  addUser,
  removeUser,
  clearDbUsers,
  clearAllDbUsers,
  isDbRoleType,
} = useDbUsersStore();
describe("dbUsers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearAllDbUsers();
  });
  it("should fetch and get users", async () => {
    db_user_list.mockResolvedValue({
      data: [
        {
          username: "testUser",
          role: "read",
        },
        {
          username: "testOwner",
          role: "admin",
        },
      ],
    });
    expect(getDbUsers(dbIdentification)).toBeUndefined();

    await fetchDbUsers(dbIdentification);
    expect(db_user_list).toHaveBeenCalledOnce();
    const users = getDbUsers(dbIdentification);
    expect(users).toHaveLength(2);
    expect(users?.[0].username).toBe("testUser");
  });
  it("should add user", async () => {
    await addUser({
      ...dbIdentification,
      username: "testUser",
      db_role: "read",
    });
    expect(db_user_add).toHaveBeenCalledOnce();
  });
  it("should remove user", async () => {
    await removeUser({
      ...dbIdentification,
      username: "testUser",
    });
    expect(db_user_remove).toHaveBeenCalledOnce();
  });
  it("should clear users", () => {
    clearDbUsers(dbIdentification);
    expect(getDbUsers(dbIdentification)).toBeUndefined();
  });
  it("should check role type", () => {
    expect(isDbRoleType("read")).toBe(true);
    expect(isDbRoleType("write")).toBe(true);
    expect(isDbRoleType("admin")).toBe(true);
    expect(isDbRoleType("other")).toBe(false);
  });
});
