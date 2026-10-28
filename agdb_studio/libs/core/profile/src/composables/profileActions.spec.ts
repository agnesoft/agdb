import { vi, describe, it, beforeEach, expect } from "vitest";
import type {
  ActionProps,
  Input,
} from "@agdb-studio/common/src/composables/content/types";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";

const {
  isAdmin,
  isAdminView,
  pushMock,
  openModal,
  sessions,
  fetchUserStatus,
  logoutSession,
} = vi.hoisted(() => {
  return {
    isAdmin: { value: true },
    isAdminView: { value: false },
    pushMock: vi.fn(),
    openModal: vi.fn(),
    fetchUserStatus: vi.fn().mockResolvedValue(undefined),
    logoutSession: vi.fn().mockResolvedValue(undefined),
    sessions: {
      value: [
        {
          agent: "browser",
          created: 1000,
          expires_at: 2000,
          session: "s1",
        },
        {
          agent: "cli",
          created: 1100,
          expires_at: 2100,
          session: "s2",
        },
      ],
    },
  };
});

vi.mock("./admin", () => {
  return {
    useAdmin: vi.fn().mockReturnValue({
      isAdmin,
      isAdminView,
    }),
  };
});

vi.mock("@agdb-studio/auth/src/auth", () => {
  return {
    useAuth: vi.fn().mockReturnValue({
      logoutSession,
    }),
  };
});

vi.mock("@agdb-studio/auth/src/account", () => {
  return {
    useAccount: vi.fn().mockReturnValue({
      sessions,
      fetchUserStatus,
    }),
  };
});

vi.mock("@agdb-studio/common/src/composables/modal/modal", () => ({
  default: () => ({ openModal }),
}));

vi.mock("@agdb-studio/router/src/router", () => {
  return {
    getRouter: vi.fn().mockReturnValue({
      push: pushMock,
    }),
  };
});

const {
  useUserActions,
  ADMIN_VIEW_KEY,
  USER_VIEW_KEY,
  SESSIONS_KEY,
  CHANGE_PASSWORD_KEY,
  LOGOUT_KEY,
} = await import("./profileActions");

describe("profileActions.ts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("returns the profile actions when admin in user screen", () => {
    isAdmin.value = true;
    isAdminView.value = false;
    const { actions } = useUserActions();
    expect(actions.value.length).toBe(4);
    expect(actions.value.some((action) => action.key === ADMIN_VIEW_KEY)).toBe(
      true,
    );
    expect(actions.value.some((action) => action.key === USER_VIEW_KEY)).toBe(
      false,
    );
    expect(actions.value.some((action) => action.key === SESSIONS_KEY)).toBe(
      true,
    );
    expect(
      actions.value.some((action) => action.key === CHANGE_PASSWORD_KEY),
    ).toBe(true);
    expect(actions.value.some((action) => action.key === LOGOUT_KEY)).toBe(
      true,
    );
  });

  it("returns the profile actions when admin in admin screen", () => {
    isAdmin.value = true;
    isAdminView.value = true;
    const { actions } = useUserActions();
    expect(actions.value.length).toBe(4);
    expect(actions.value.some((action) => action.key === ADMIN_VIEW_KEY)).toBe(
      false,
    );
    expect(actions.value.some((action) => action.key === USER_VIEW_KEY)).toBe(
      true,
    );
    expect(actions.value.some((action) => action.key === SESSIONS_KEY)).toBe(
      true,
    );
    expect(
      actions.value.some((action) => action.key === CHANGE_PASSWORD_KEY),
    ).toBe(true);
    expect(actions.value.some((action) => action.key === LOGOUT_KEY)).toBe(
      true,
    );
  });

  it("returns the profile actions when not admin in user screen", () => {
    isAdmin.value = false;
    isAdminView.value = false;
    const { actions } = useUserActions();
    expect(actions.value.length).toBe(3);
    expect(actions.value.some((action) => action.key === ADMIN_VIEW_KEY)).toBe(
      false,
    );
    expect(actions.value.some((action) => action.key === USER_VIEW_KEY)).toBe(
      false,
    );
    expect(actions.value.some((action) => action.key === SESSIONS_KEY)).toBe(
      true,
    );
    expect(
      actions.value.some((action) => action.key === CHANGE_PASSWORD_KEY),
    ).toBe(true);
    expect(actions.value.some((action) => action.key === LOGOUT_KEY)).toBe(
      true,
    );
  });

  it("links to the user view", () => {
    isAdmin.value = true;
    isAdminView.value = true;
    const { actions } = useUserActions();
    actions.value
      .find((action) => action.key === USER_VIEW_KEY)
      ?.action?.({} as ActionProps<undefined>);
    expect(pushMock).toHaveBeenCalledWith({ name: "home" });
  });

  it("links to the admin view", () => {
    isAdmin.value = true;
    isAdminView.value = false;
    const { actions } = useUserActions();
    actions.value
      .find((action) => action.key === ADMIN_VIEW_KEY)
      ?.action?.({} as ActionProps<undefined>);
    expect(pushMock).toHaveBeenCalledWith({ name: "admin" });
  });

  it("opens sessions modal with session table and row actions", async () => {
    const { actions } = useUserActions();
    await actions.value
      .find((action) => action.key === SESSIONS_KEY)
      ?.action?.({} as ActionProps<undefined>);
    expect(fetchUserStatus).toHaveBeenCalled();
    expect(openModal).toHaveBeenCalledWith(
      expect.objectContaining({
        header: "Active Sessions",
        wide: true,
      }),
    );
    const content = openModal.mock.calls[0]?.[0]?.content;
    expect(content).toHaveLength(1);
    expect(content[0]?.table?.headers).toEqual(["Agent", "Created", "Expires"]);
    expect(content[0]?.table?.rows).toHaveLength(2);
    expect(content[0]?.table?.rowActions).toHaveLength(1);
    expect(content[0]?.table?.rowActions[0]?.label).toBe("Logout");
  });

  it("logs out a session from sessions modal and refreshes", async () => {
    const { actions } = useUserActions();
    await actions.value
      .find((action) => action.key === SESSIONS_KEY)
      ?.action?.({} as ActionProps<undefined>);
    const content = openModal.mock.calls[0]?.[0]?.content;
    const onLogout = content[0]?.table?.rowActions[0]?.onClick;
    await onLogout?.(1);
    expect(logoutSession).toHaveBeenCalledWith(true, "s2");
    expect(fetchUserStatus).toHaveBeenCalledTimes(2);
    expect(openModal).toHaveBeenCalledTimes(2);
  });

  it("handles fetchUserStatus failure in sessions modal row logout", async () => {
    const { actions } = useUserActions();
    await actions.value
      .find((action) => action.key === SESSIONS_KEY)
      ?.action?.({} as ActionProps<undefined>);
    fetchUserStatus.mockRejectedValueOnce(new Error("fail"));
    const content = openModal.mock.calls[0]?.[0]?.content;
    const onLogout = content[0]?.table?.rowActions[0]?.onClick;
    await onLogout?.(0);
    expect(logoutSession).toHaveBeenCalledWith(true, "s1");
    expect(openModal).toHaveBeenCalledTimes(2);
  });

  it("ignores out-of-range index in sessions modal row logout", async () => {
    const { actions } = useUserActions();
    await actions.value
      .find((action) => action.key === SESSIONS_KEY)
      ?.action?.({} as ActionProps<undefined>);
    const content = openModal.mock.calls[0]?.[0]?.content;
    const onLogout = content[0]?.table?.rowActions[0]?.onClick;
    await onLogout?.(99);
    expect(logoutSession).not.toHaveBeenCalled();
  });

  it("shows no sessions message when empty", async () => {
    sessions.value = [];
    const { actions } = useUserActions();
    await actions.value
      .find((action) => action.key === SESSIONS_KEY)
      ?.action?.({} as ActionProps<undefined>);
    const content = openModal.mock.calls[0]?.[0]?.content;
    expect(content).toHaveLength(1);
    expect(content[0]?.paragraph?.[0]?.text).toBe("No active sessions.");
    sessions.value = [
      { agent: "browser", created: 1000, expires_at: 2000, session: "s1" },
      { agent: "cli", created: 1100, expires_at: 2100, session: "s2" },
    ];
  });

  describe("logout with session management", () => {
    const { addInput, setInputValue, clearAllInputs } = useContentInputs();

    const clusterInput: Input = {
      type: "checkbox",
      label: "Cluster",
      key: "cluster",
    };

    const sessionInput: Input = {
      type: "select",
      label: "Session",
      key: "session",
    };

    beforeEach(() => {
      clearAllInputs();
      sessions.value = [
        { agent: "browser", created: 1000, expires_at: 2000, session: "s1" },
        { agent: "cli", created: 1100, expires_at: 2100, session: "s2" },
      ];
    });

    it("fetches user status and shows session selector when multiple sessions", async () => {
      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);
      expect(fetchUserStatus).toHaveBeenCalled();
      const content = openModal.mock.calls[0]?.[0]?.content;
      const selectContent = content?.find(
        (c: { input?: { key: string } }) => c.input?.key === "session",
      );
      expect(selectContent).toBeDefined();
      expect(selectContent?.input?.options).toHaveLength(5);
      expect(selectContent?.input?.options[0]?.label).toBe("Current session");
      expect(selectContent?.input?.options[1]?.label).toBe("All sessions");
      expect(selectContent?.input?.options[2]?.label).toBe("Other sessions");
      expect(selectContent?.input?.options[3]?.label).toContain("browser");
      expect(selectContent?.input?.options[4]?.label).toContain("cli");
    });

    it("omits session selector when single session", async () => {
      sessions.value = [
        { agent: "browser", created: 1000, expires_at: 2000, session: "s1" },
      ];
      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);
      const content = openModal.mock.calls[0]?.[0]?.content;
      const selectContent = content?.find(
        (c: { input?: { key: string } }) => c.input?.key === "session",
      );
      expect(selectContent).toBeUndefined();
    });

    it("logs out current session by default", async () => {
      addInput(KEY_MODAL, clusterInput);
      setInputValue(KEY_MODAL, clusterInput.key, false);
      addInput(KEY_MODAL, sessionInput);
      setInputValue(KEY_MODAL, sessionInput.key, "");

      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(logoutSession).toHaveBeenCalledWith(false, undefined);
    });

    it("logs out all sessions", async () => {
      addInput(KEY_MODAL, clusterInput);
      setInputValue(KEY_MODAL, clusterInput.key, true);
      addInput(KEY_MODAL, sessionInput);
      setInputValue(KEY_MODAL, sessionInput.key, "all");

      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(logoutSession).toHaveBeenCalledWith(true, "all");
    });

    it("logs out other sessions without ending current session", async () => {
      addInput(KEY_MODAL, clusterInput);
      setInputValue(KEY_MODAL, clusterInput.key, false);
      addInput(KEY_MODAL, sessionInput);
      setInputValue(KEY_MODAL, sessionInput.key, "others");

      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(logoutSession).toHaveBeenCalledWith(false, "others");
      expect(fetchUserStatus).toHaveBeenCalledTimes(2);
    });

    it("logs out a specific session", async () => {
      addInput(KEY_MODAL, clusterInput);
      setInputValue(KEY_MODAL, clusterInput.key, false);
      addInput(KEY_MODAL, sessionInput);
      setInputValue(KEY_MODAL, sessionInput.key, "s2");

      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(logoutSession).toHaveBeenCalledWith(false, "s2");
      expect(fetchUserStatus).toHaveBeenCalledTimes(2);
    });

    it("logs out a specific session with cluster", async () => {
      addInput(KEY_MODAL, clusterInput);
      setInputValue(KEY_MODAL, clusterInput.key, true);
      addInput(KEY_MODAL, sessionInput);
      setInputValue(KEY_MODAL, sessionInput.key, "s1");

      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);

      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      await onConfirm?.();
      expect(logoutSession).toHaveBeenCalledWith(true, "s1");
    });

    it("handles fetchUserStatus failure after specific session logout", async () => {
      addInput(KEY_MODAL, clusterInput);
      setInputValue(KEY_MODAL, clusterInput.key, false);
      addInput(KEY_MODAL, sessionInput);
      setInputValue(KEY_MODAL, sessionInput.key, "s2");

      const { actions } = useUserActions();
      await actions.value
        .find((action) => action.key === LOGOUT_KEY)
        ?.action?.({} as ActionProps<undefined>);

      fetchUserStatus.mockRejectedValueOnce(new Error("fail"));
      const onConfirm = openModal.mock.calls[0]?.[0]?.onConfirm;
      const result = await onConfirm?.();
      expect(logoutSession).toHaveBeenCalledWith(false, "s2");
      expect(result).toBe(true);
    });
  });
});
