import { vi, describe, it, beforeEach, expect } from "vitest";
import {
  useUserActions,
  ADMIN_VIEW_KEY,
  USER_VIEW_KEY,
  CHANGE_PASSWORD_KEY,
  LOGOUT_KEY,
} from "./profileActions";
import type { ActionProps } from "@agdb-studio/common/src/composables/content/types";

const { isAdmin, isAdminView, pushMock } = vi.hoisted(() => {
  return {
    isAdmin: { value: true },
    isAdminView: { value: false },
    pushMock: vi.fn(),
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

vi.mock("@agdb-studio/router/src/router", () => {
  return {
    getRouter: vi.fn().mockReturnValue({
      push: pushMock,
    }),
  };
});

describe("profileActions.ts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("returns the profile actions when admin in user screen", () => {
    isAdmin.value = true;
    isAdminView.value = false;
    const { actions } = useUserActions();
    expect(actions.value.length).toBe(3);
    expect(actions.value.some((action) => action.key === ADMIN_VIEW_KEY)).toBe(
      true,
    );
    expect(actions.value.some((action) => action.key === USER_VIEW_KEY)).toBe(
      false,
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
    expect(actions.value.length).toBe(3);
    expect(actions.value.some((action) => action.key === ADMIN_VIEW_KEY)).toBe(
      false,
    );
    expect(actions.value.some((action) => action.key === USER_VIEW_KEY)).toBe(
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
    expect(actions.value.length).toBe(2);
    expect(actions.value.some((action) => action.key === ADMIN_VIEW_KEY)).toBe(
      false,
    );
    expect(actions.value.some((action) => action.key === USER_VIEW_KEY)).toBe(
      false,
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
});
