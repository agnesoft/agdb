import { describe, beforeEach, vi, it, expect } from "vitest";

import * as dbActions from "@/composables/db/dbActions";

import {
  db_list,
  db_add,
  db_backup,
  db_restore,
  db_clear,
  db_convert,
  db_remove,
  db_delete,
  db_optimize,
  db_audit,
  db_copy,
  db_rename,
  db_user_list,
  db_user_add,
  db_user_remove,
  db_exec,
  db_exec_mut,
  admin_db_list,
  admin_db_add,
  admin_db_backup,
  admin_db_restore,
  admin_db_clear,
  admin_db_convert,
  admin_db_remove,
  admin_db_delete,
  admin_db_optimize,
  admin_db_audit,
  admin_db_copy,
  admin_db_rename,
  admin_db_user_list,
  admin_db_user_add,
  admin_db_user_remove,
  admin_db_exec,
  admin_db_exec_mut,
} from "@/tests/apiMock";

const { username, isAdmin, isAdminView } = vi.hoisted(() => {
  return {
    username: { value: "test_user" },
    isAdmin: { value: true },
    isAdminView: { value: false },
  };
});
vi.mock("@/composables/profile/account", () => {
  return {
    useAccount: () => {
      return {
        username,
      };
    },
  };
});

vi.mock("@/composables/profile/admin", () => {
  return {
    useAdmin: vi.fn().mockReturnValue({
      isAdmin,
      isAdminView,
    }),
  };
});

describe("dbActions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  describe("user actions", () => {
    beforeEach(() => {
      isAdmin.value = false;
      isAdminView.value = false;
    });
    it("should add a database", async () => {
      dbActions.dbAdd({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
      expect(db_add).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
    });

    it("should list databases", async () => {
      dbActions.dbList();
      expect(db_list).toHaveBeenCalled();
    });

    it("should backup a database", async () => {
      dbActions.dbBackup({ owner: "test_user", db: "test_db" });
      expect(db_backup).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should restore a database", async () => {
      dbActions.dbRestore({ owner: "test_user", db: "test_db" });
      expect(db_restore).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should clear a database", async () => {
      dbActions.dbClear({
        owner: "test_user",
        db: "test_db",
        resource: "db",
      });
      expect(db_clear).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        resource: "db",
      });
    });

    it("should convert a database", async () => {
      dbActions.dbConvert({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
      expect(db_convert).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
    });

    it("should remove a database", async () => {
      dbActions.dbRemove({ owner: "test_user", db: "test_db" });
      expect(db_remove).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should delete a database", async () => {
      dbActions.dbDelete({ owner: "test_user", db: "test_db" });
      expect(db_delete).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should optimize a database", async () => {
      dbActions.dbOptimize({ owner: "test_user", db: "test_db" });
      expect(db_optimize).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should audit a database", async () => {
      dbActions.dbAudit({ owner: "test_user", db: "test_db" });
      expect(db_audit).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should copy a database", async () => {
      dbActions.dbCopy({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
        new_owner: "new_user",
      });
      expect(db_copy).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
      });
    });

    it("should rename a database", async () => {
      dbActions.dbRename({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
        new_owner: "",
      });
      expect(db_rename).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
      });
    });

    it("should list users", async () => {
      dbActions.dbUserList({ owner: "test_user", db: "test_db" });
      expect(db_user_list).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should add a user", async () => {
      dbActions.dbUserAdd({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
        db_role: "read",
      });
      expect(db_user_add).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
        db_role: "read",
      });
    });

    it("should remove a user", async () => {
      dbActions.dbUserRemove({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
      });
      expect(db_user_remove).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
      });
    });

    it("should execute a query", async () => {
      dbActions.dbExec({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
      expect(db_exec).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
    });

    it("should execute a mutation", async () => {
      dbActions.dbExecMut({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
      expect(db_exec_mut).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
    });
  });

  describe("admin actions", () => {
    beforeEach(() => {
      isAdminView.value = true;
      isAdmin.value = true;
    });
    it("should list databases", async () => {
      dbActions.dbList();
      expect(admin_db_list).toHaveBeenCalled();
    });

    it("should add a database", async () => {
      dbActions.dbAdd({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
      expect(admin_db_add).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
    });

    it("should backup a database", async () => {
      dbActions.dbBackup({ owner: "test_user", db: "test_db" });
      expect(admin_db_backup).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should restore a database", async () => {
      dbActions.dbRestore({ owner: "test_user", db: "test_db" });
      expect(admin_db_restore).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should clear a database", async () => {
      dbActions.dbClear({
        owner: "test_user",
        db: "test_db",
        resource: "db",
      });
      expect(admin_db_clear).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        resource: "db",
      });
    });

    it("should convert a database", async () => {
      dbActions.dbConvert({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
      expect(admin_db_convert).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        db_type: "memory",
      });
    });

    it("should remove a database", async () => {
      dbActions.dbRemove({ owner: "test_user", db: "test_db" });
      expect(admin_db_remove).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should delete a database", async () => {
      dbActions.dbDelete({ owner: "test_user", db: "test_db" });
      expect(admin_db_delete).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should optimize a database", async () => {
      dbActions.dbOptimize({ owner: "test_user", db: "test_db" });
      expect(admin_db_optimize).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should audit a database", async () => {
      dbActions.dbAudit({ owner: "test_user", db: "test_db" });
      expect(admin_db_audit).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should copy a database", async () => {
      dbActions.dbCopy({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
        new_owner: "new_user",
      });
      expect(admin_db_copy).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
        new_owner: "new_user",
      });
    });

    it("should rename a database", async () => {
      dbActions.dbRename({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
        new_owner: "new_user",
      });
      expect(admin_db_rename).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        new_db: "new_db",
        new_owner: "new_user",
      });
    });

    it("should list users", async () => {
      dbActions.dbUserList({ owner: "test_user", db: "test_db" });
      expect(admin_db_user_list).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
      });
    });

    it("should add a user", async () => {
      dbActions.dbUserAdd({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
        db_role: "read",
      });
      expect(admin_db_user_add).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
        db_role: "read",
      });
    });

    it("should remove a user", async () => {
      dbActions.dbUserRemove({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
      });
      expect(admin_db_user_remove).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        username: "new_user",
      });
    });

    it("should execute a query", async () => {
      dbActions.dbExec({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
      expect(admin_db_exec).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
    });

    it("should execute a mutation", async () => {
      dbActions.dbExecMut({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
      expect(admin_db_exec_mut).toHaveBeenCalledWith({
        owner: "test_user",
        db: "test_db",
        sql: "some query",
      });
    });
  });
});
