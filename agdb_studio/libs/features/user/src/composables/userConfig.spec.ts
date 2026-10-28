import { vi, describe, it, beforeEach, expect } from "vitest";
import { ref } from "vue";
import {
  admin_user_change_password,
  admin_user_logout,
  admin_user_delete,
  cluster_admin_user_logout,
} from "@agdb-studio/testing/mocks/apiMock";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import type { UserStatus } from "@agnesoft/agdb_api/openapi";
import type {
  ActionProps,
  Input,
} from "@agdb-studio/common/src/composables/content/types";

const mockUsers = ref<UserStatus[]>([
  {
    username: "test_user",
    admin: false,
    login: true,
    sessions: [
      { agent: "browser", created: 1000, expires_at: 2000, session: "s1" },
      { agent: "cli", created: 1100, expires_at: 2100, session: "s2" },
    ],
  },
]);

const openModal = vi.fn();
const fetchUsers = vi.fn().mockResolvedValue(undefined);

vi.mock("./userStore", () => ({
  useUserStore: () => ({ users: mockUsers, fetchUsers }),
}));

vi.mock("@agdb-studio/common/src/composables/modal/modal", () => ({
  default: () => ({ openModal }),
}));

const { userActions, userColumns } = await import("./userConfig");

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

const sessionInput: Input = {
  type: "select",
  label: "Session",
  key: "session",
};

describe("userConfig.ts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearAllInputs();
    mockUsers.value = [
      {
        username: "test_user",
        admin: false,
        login: true,
        sessions: [
          { agent: "browser", created: 1000, expires_at: 2000, session: "s1" },
          { agent: "cli", created: 1100, expires_at: 2100, session: "s2" },
        ],
      },
    ];
  });

  describe("userColumns", () => {
    it("returns the user columns", () => {
      expect(userColumns.length).toBe(3);
      expect(userColumns.some((column) => column.key === "username")).toBe(
        true,
      );
      expect(userColumns.some((column) => column.key === "admin")).toBe(false);
      expect(userColumns.some((column) => column.key === "sessionCount")).toBe(
        true,
      );
    });

    it("shows crown icon only for admin user", () => {
      const usernameColumn = userColumns.find(
        (column) => column.key === "username",
      );
      expect(usernameColumn).toBeDefined();

      const adminIcon =
        typeof usernameColumn?.iconResolver === "function"
          ? usernameColumn.iconResolver({
              username: "admin",
              admin: true,
              login: true,
            })
          : undefined;
      const regularIcon =
        typeof usernameColumn?.iconResolver === "function"
          ? usernameColumn.iconResolver({
              username: "user",
              admin: false,
              login: true,
            })
          : undefined;

      expect(adminIcon).toBeDefined();
      expect(regularIcon).toBeUndefined();
    });

    it("resolves crown title and class only for admin user", () => {
      const usernameColumn = userColumns.find(
        (column) => column.key === "username",
      );
      expect(usernameColumn).toBeDefined();

      const adminTitle =
        typeof usernameColumn?.iconTitleResolver === "function"
          ? usernameColumn.iconTitleResolver({
              username: "admin",
              admin: true,
              login: true,
            })
          : undefined;
      const regularTitle =
        typeof usernameColumn?.iconTitleResolver === "function"
          ? usernameColumn.iconTitleResolver({
              username: "user",
              admin: false,
              login: true,
            })
          : undefined;

      const adminClass =
        typeof usernameColumn?.iconClassResolver === "function"
          ? usernameColumn.iconClassResolver({
              username: "admin",
              admin: true,
              login: true,
            })
          : undefined;
      const regularClass =
        typeof usernameColumn?.iconClassResolver === "function"
          ? usernameColumn.iconClassResolver({
              username: "user",
              admin: false,
              login: true,
            })
          : undefined;

      expect(adminTitle).toBe("Admin user");
      expect(regularTitle).toBeUndefined();
      expect(adminClass).toBe("crown-icon");
      expect(regularClass).toBe("");
    });
  });

  describe("userActions", () => {
    it("returns the user actions", () => {
      expect(userActions.length).toBe(4);
      expect(userActions.some((action) => action.key === "view_sessions")).toBe(
        true,
      );
      expect(
        userActions.some((action) => action.key === "change_password"),
      ).toBe(true);
      expect(userActions.some((action) => action.key === "logout")).toBe(true);
      expect(userActions.some((action) => action.key === "delete")).toBe(true);
    });

    it("opens modal with session details for view_sessions action", async () => {
      const action = userActions.find(
        (action) => action.key === "view_sessions",
      );
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);
      expect(fetchUsers).toHaveBeenCalled();
      expect(openModal).toHaveBeenCalledWith(
        expect.objectContaining({
          header: "Sessions for test_user",
          wide: true,
        }),
      );
      const content = openModal.mock.calls[0]?.[0]?.content;
      expect(content).toHaveLength(1);
      expect(content[0]?.table?.headers).toEqual([
        "Agent",
        "Created",
        "Expires",
      ]);
      expect(content[0]?.table?.rows).toHaveLength(2);
    });

    it("logs out a session from sessions modal and refreshes", async () => {
      const action = userActions.find(
        (action) => action.key === "view_sessions",
      );
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);
      const content = openModal.mock.calls[0]?.[0]?.content;
      const onLogout = content[0]?.table?.rowActions[0]?.onClick;
      await onLogout?.(1);
      expect(admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
        session: "s2",
      });
      expect(fetchUsers).toHaveBeenCalledTimes(2);
      expect(openModal).toHaveBeenCalledTimes(2);
    });

    it("ignores out-of-range index in sessions modal row logout", async () => {
      const action = userActions.find(
        (action) => action.key === "view_sessions",
      );
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);
      const content = openModal.mock.calls[0]?.[0]?.content;
      const onLogout = content[0]?.table?.rowActions[0]?.onClick;
      await onLogout?.(99);
      expect(admin_user_logout).not.toHaveBeenCalled();
    });

    it("handles fetchUsers failure in sessions modal row logout", async () => {
      const action = userActions.find(
        (action) => action.key === "view_sessions",
      );
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);
      fetchUsers.mockRejectedValueOnce(new Error("fail"));
      const content = openModal.mock.calls[0]?.[0]?.content;
      const onLogout = content[0]?.table?.rowActions[0]?.onClick;
      await onLogout?.(0);
      expect(admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
        session: "s1",
      });
      expect(openModal).toHaveBeenCalledTimes(2);
    });

    it("shows no sessions message for user without sessions", async () => {
      mockUsers.value = [
        {
          username: "empty_user",
          admin: false,
          login: false,
          sessions: [],
        },
      ];
      const action = userActions.find(
        (action) => action.key === "view_sessions",
      );
      await action?.action?.({
        params: { username: "empty_user" },
      } as ActionProps<UserStatus>);
      const content = openModal.mock.calls[0]?.[0]?.content;
      expect(content[0]?.paragraph?.[0]?.text).toBe("No active sessions.");
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
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);

      expect(openModal).toHaveBeenCalledWith(
        expect.objectContaining({
          header: "Logout test_user",
        }),
      );

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
      });
    });

    it("executes the logout cluster action", async () => {
      addInput(KEY_MODAL, checkboxInput);
      setInputValue(KEY_MODAL, checkboxInput.key, true);

      const action = userActions.find((action) => action.key === "logout");
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(cluster_admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
      });
    });

    it("builds logout modal with session select when multiple sessions", async () => {
      const action = userActions.find((action) => action.key === "logout");
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);

      const content = openModal.mock.calls[0]?.[0]?.content;
      const selectContent = content?.find(
        (c: { input?: { key: string } }) => c.input?.key === "session",
      );
      expect(selectContent).toBeDefined();
      expect(selectContent?.input?.options).toHaveLength(3);
    });

    it("builds logout modal without session select when single session", async () => {
      mockUsers.value = [
        {
          username: "test_user",
          admin: false,
          login: true,
          sessions: [
            {
              agent: "browser",
              created: 1000,
              expires_at: 2000,
              session: "s1",
            },
          ],
        },
      ];
      const action = userActions.find((action) => action.key === "logout");
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);

      const content = openModal.mock.calls[0]?.[0]?.content;
      const selectContent = content?.find(
        (c: { input?: { key: string } }) => c.input?.key === "session",
      );
      expect(selectContent).toBeUndefined();
    });

    it("executes logout for a specific session", async () => {
      addInput(KEY_MODAL, checkboxInput);
      setInputValue(KEY_MODAL, checkboxInput.key, false);
      addInput(KEY_MODAL, sessionInput);
      setInputValue(KEY_MODAL, sessionInput.key, "s1");

      const action = userActions.find((action) => action.key === "logout");
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
        session: "s1",
      });
    });

    it("handles fetchUsers failure after logout confirm", async () => {
      addInput(KEY_MODAL, checkboxInput);
      setInputValue(KEY_MODAL, checkboxInput.key, false);

      const action = userActions.find((action) => action.key === "logout");
      await action?.action?.({
        params: { username: "test_user" },
      } as ActionProps<UserStatus>);

      fetchUsers.mockRejectedValueOnce(new Error("fail"));
      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      const result = await onConfirm?.();
      expect(admin_user_logout).toHaveBeenCalledWith({
        username: "test_user",
      });
      expect(result).toBe(true);
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
