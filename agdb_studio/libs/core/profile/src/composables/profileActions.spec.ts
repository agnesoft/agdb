import { vi, describe, it, beforeEach, expect } from "vitest";
import type { ActionProps } from "@agdb-studio/common/src/composables/content/types";

const { isAdmin, isAdminView, pushMock, openModal, sessions, fetchUserStatus } =
  vi.hoisted(() => {
    return {
      isAdmin: { value: true },
      isAdminView: { value: false },
      pushMock: vi.fn(),
      openModal: vi.fn(),
      fetchUserStatus: vi.fn().mockResolvedValue(undefined),
      sessions: {
        value: [
          {
            agent: "browser",
            created: 1000,
            expires_at: 2000,
            session: "s1",
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

  it("opens sessions modal with session table", async () => {
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
    expect(content[0]?.table?.rows).toHaveLength(1);
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
    ];
  });
});
