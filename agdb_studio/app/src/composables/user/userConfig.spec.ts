import { vi, describe, it, beforeEach, expect } from "vitest";
import { userActions, userColumns } from "./userConfig";
import {
  admin_user_change_password,
  admin_user_logout,
  admin_user_delete,
  cluster_admin_user_logout,
} from "@agdb-studio/testing/mocks/apiMock";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "../modal/constants";
import type { UserStatus } from "@agnesoft/agdb_api/openapi";
const { addInput, setInputValue, clearAllInputs } = useContentInputs();

const passwordInput: Input = {
  type: "text",
  label: "Password",
  key: "password",
};

const checkboxInput: Input = {
  type: "checkbox",
  label: "Cluster",
  key: "cluster",
};

describe("userConfig.ts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearAllInputs();
  });

  describe("userColumns", () => {
    it("returns the user columns", () => {
      expect(userColumns.length).toBe(4);
      expect(userColumns.some((column) => column.key === "username")).toBe(
        true,
      );
      expect(userColumns.some((column) => column.key === "admin")).toBe(true);
      expect(userColumns.some((column) => column.key === "login")).toBe(true);
    });
  });

  describe("userActions", () => {
    it("returns the user actions", () => {
      expect(userActions.length).toBe(3);
      expect(
        userActions.some((action) => action.key === "change_password"),
      ).toBe(true);
      expect(userActions.some((action) => action.key === "logout")).toBe(true);
      expect(userActions.some((action) => action.key === "delete")).toBe(true);
    });

    it("executes the change password action", async () => {
      addInput(KEY_MODAL, passwordInput);
      setInputValue(KEY_MODAL, passwordInput.key, "new_password");
      const action = userActions.find(
        (action) => action.key === "change_password",
      );
      const header =
        typeof action?.confirmationHeader === "function"
          ? action?.confirmationHeader({
              params: { username: "test_user" },
            } as unknown as ActionProps<UserStatus>)
          : "";
      expect(header).toBe("Change password for test_user");
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);
      expect(admin_user_change_password).toHaveBeenCalledWith(
        { username: "test_user" },
        { password: "new_password" },
      );
    });

    it("executes the logout action", async () => {
      addInput(KEY_MODAL, checkboxInput);
      setInputValue(KEY_MODAL, checkboxInput.key, false);

      const action = userActions.find((action) => action.key === "logout");
      const header =
        typeof action?.confirmationHeader === "function"
          ? action?.confirmationHeader({
              params: { username: "test_user" },
            } as unknown as ActionProps<UserStatus>)
          : "";

      expect(header).toBe("Logout test_user");

      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);
      expect(admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
      });
    });

    it("executes the logout cluster action", async () => {
      addInput(KEY_MODAL, checkboxInput);
      setInputValue(KEY_MODAL, checkboxInput.key, true);
      await userActions
        .find((action) => action.key === "logout")
        ?.action?.({
          params: { username: "test_user" },
        } as ActionProps<UserStatus>);
      expect(cluster_admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
      });
    });

    it("executes the delete action", async () => {
      const action = userActions.find((action) => action.key === "delete");

      const header =
        typeof action?.confirmationHeader === "function"
          ? action?.confirmationHeader({
              params: { username: "test_user" },
            } as unknown as ActionProps<UserStatus>)
          : "";
      expect(header).toBe("Delete test_user");
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);
      expect(admin_user_delete).toHaveBeenCalledWith({
        username: "test_user",
      });
    });
  });
});
