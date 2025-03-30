import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  dbActions,
  dbColumns,
  getConfirmationHeaderFn,
  type DbActionProps,
} from "./dbConfig";
import {
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
  admin_db_copy,
  admin_db_rename,
} from "@agdb-studio/testing/mocks/apiMock";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "../modal/constants";
import useModal from "../modal/modal";
import type { ServerDatabase } from "@agnesoft/agdb_api/openapi";

const { isAdmin, isAdminView } = vi.hoisted(() => {
  return {
    isAdmin: { value: true },
    isAdminView: { value: false },
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

const { addInput, setInputValue, clearAllInputs } = useContentInputs();

const { modalIsVisible, modal } = useModal();

const testInput: Input = {
  key: "new_db",
  label: "Test label",
  type: "text",
  autofocus: true,
  required: true,
};
const testOwnerInput: Input = {
  key: "new_owner",
  label: "Test label",
  type: "text",
  autofocus: false,
  required: true,
};

describe("dbConfig", () => {
  describe("dbColumns", () => {
    it("should have correct db columns", () => {
      const columnKeys = dbColumns.map((column) => column.key);
      expect(columnKeys).toContain("owner");
      expect(columnKeys).toContain("db");
      expect(columnKeys).toContain("db_type");
      expect(columnKeys).toContain("size");
      expect(columnKeys).toContain("backup");
    });
  });
  describe("dbActions", () => {
    beforeEach(() => {
      clearAllInputs();
      vi.clearAllMocks();
      isAdminView.value = false;
    });
    it("should have correct db actions", () => {
      const actionKeys = dbActions.map((action) => action.key);
      expect(actionKeys).toContain("backup");
      expect(actionKeys).toContain("optimize");
      expect(actionKeys).toContain("audit");
      expect(actionKeys).toContain("remove");
      expect(actionKeys).toContain("delete");
      expect(actionKeys).toContain("restore");
    });
    it.each([
      ["backup", db_backup],
      ["restore", db_restore],
      ["remove", db_remove],
      ["delete", db_delete],
      ["optimize", db_optimize],
      ["audit", db_audit],
    ])("should run correct db actions for %s", (key, api) => {
      const action = dbActions.find((action) => action.key === key);
      const params = {
        db: "test_db",
        owner: "test_owner",
      };
      action?.action?.({ params } as ActionProps<ServerDatabase>);
      expect(api).toHaveBeenCalledWith(params);
    });

    it.each([
      ["clear", "all", "resource", db_clear],
      ["clear", "db", "resource", db_clear],
      ["clear", "audit", "resource", db_clear],
      ["clear", "backup", "resource", db_clear],
      ["convert", "file", "db_type", db_convert],
      ["convert", "memory", "db_type", db_convert],
      ["convert", "mapped", "db_type", db_convert],
    ])(
      "should run correct db actions for %s with %s",
      (key, value, valueKey, api) => {
        const action = dbActions.find((action) => action.key === key);
        const subaction = action?.actions?.find(
          (action) => action.key === value,
        );
        const params = { db: "test_db" };
        subaction?.action({ params });
        expect(api).toHaveBeenCalledWith({
          ...params,
          [valueKey]: value,
        });
      },
    );

    it.each([
      ["copy", db_copy],
      ["rename", db_rename],
    ])("should run correct db actions for %s as user", (key, api) => {
      const newName = "new_test_db";
      addInput(KEY_MODAL, testInput);
      setInputValue(KEY_MODAL, testInput.key, newName);
      const action = dbActions.find((action) => action.key === key);
      const params = { db: "test_db" };
      action?.action?.({ params } as ActionProps<ServerDatabase>);
      expect(api).toHaveBeenCalledWith({
        ...params,
        new_db: newName,
      });
      clearAllInputs();
    });

    it.each([
      ["copy", admin_db_copy],
      ["rename", admin_db_rename],
    ])("should run correct db actions for %s as admin", (key, api) => {
      isAdmin.value = true;
      isAdminView.value = true;
      const newName = "new_test_db";
      addInput(KEY_MODAL, testInput);
      setInputValue(KEY_MODAL, testInput.key, newName);
      const newOwner = "new_owner";
      addInput(KEY_MODAL, testOwnerInput);
      setInputValue(KEY_MODAL, testOwnerInput.key, newOwner);
      const action = dbActions.find((action) => action.key === key);
      const params = { db: "test_db" };
      action?.action?.({ params } as ActionProps<ServerDatabase>);
      expect(api).toHaveBeenCalledWith({
        ...params,
        new_db: newName,
        new_owner: newOwner,
      });
      clearAllInputs();
    });

    it("should print the empty audit log", async () => {
      const action = dbActions.find((action) => action.key === "audit");
      const params = { db: "test_db", owner: "test_owner" };
      await action?.action?.({ params } as ActionProps<ServerDatabase>);
      expect(db_audit).toHaveBeenCalledWith(params);

      expect(modalIsVisible.value).toBe(true);
      expect(modal.header).toBe("Audit log of test_owner/test_db");
      expect(modal.content).toHaveLength(1);
    });

    it("should print the audit log", async () => {
      const action = dbActions.find((action) => action.key === "audit");
      const params = { db: "test_db", owner: "test_owner" };
      db_audit.mockResolvedValueOnce({
        data: [
          {
            timestamp: "123",
            username: "test_user",
            query: "test_query",
          },
          {
            timestamp: "456",
            username: "test_user2",
            query: "test_query2",
          },
        ],
      });
      await action?.action?.({ params } as ActionProps<ServerDatabase>);
      expect(db_audit).toHaveBeenCalledWith(params);

      expect(modalIsVisible.value).toBe(true);
      expect(modal.header).toBe("Audit log of test_owner/test_db");
      expect(modal.content).toHaveLength(2);
      expect(modal.content[0].paragraph?.at(0)?.text).toBe(
        "123 | test_user | test_query",
      );
      expect(modal.content[1].paragraph?.at(0)?.text).toBe(
        "456 | test_user2 | test_query2",
      );
    });

    it("should create correct inputs for copy action for user", () => {
      const action = dbActions.find((action) => action.key === "copy");
      const params = { db: "test_db", owner: "test_owner" };

      if (typeof action?.confirmation !== "function") {
        throw new Error("Confirmation function not found");
      }

      const content = action.confirmation({
        params,
      } as ActionProps<ServerDatabase>);
      expect(content).toHaveLength(2);
      expect(content[1].input?.key).toBe("new_db");
    });

    it("should create correct inputs for copy action for admin", () => {
      isAdmin.value = true;
      isAdminView.value = true;
      const action = dbActions.find((action) => action.key === "copy");
      const params = { db: "test_db", owner: "test_owner" };

      if (typeof action?.confirmation !== "function") {
        throw new Error("Confirmation function not found");
      }

      const content = action.confirmation({
        params,
      } as ActionProps<ServerDatabase>);
      expect(content).toHaveLength(3);
      expect(content[1].input?.key).toBe("new_db");
      expect(content[2].input?.key).toBe("new_owner");
    });

    it("should create correct inputs for rename action for user", () => {
      const action = dbActions.find((action) => action.key === "rename");
      const params = { db: "test_db", owner: "test_owner" };

      if (typeof action?.confirmation !== "function") {
        throw new Error("Confirmation function not found");
      }

      const content = action.confirmation({
        params,
      } as ActionProps<ServerDatabase>);
      expect(content).toHaveLength(2);
      expect(content[1].input?.key).toBe("new_db");
    });

    it("should create correct inputs for rename action for admin", () => {
      isAdmin.value = true;
      isAdminView.value = true;
      const action = dbActions.find((action) => action.key === "rename");
      const params = { db: "test_db", owner: "test_owner" };

      if (typeof action?.confirmation !== "function") {
        throw new Error("Confirmation function not found");
      }

      const content = action.confirmation({
        params,
      } as ActionProps<ServerDatabase>);
      expect(content).toHaveLength(3);
      expect(content[1].input?.key).toBe("new_db");
      expect(content[2].input?.key).toBe("new_owner");
    });
  });

  describe("getConfirmationHeaderFn", () => {
    it("should return correct header", () => {
      const header = getConfirmationHeaderFn({
        params: { db: "test_db", owner: "test_owner" },
      } as unknown as DbActionProps);
      expect(header).toBe("Confirm action for test_owner/test_db");
    });
  });
});
