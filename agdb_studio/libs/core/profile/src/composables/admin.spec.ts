import { vi, describe, it, beforeEach, expect } from "vitest";
import { useAdmin } from "./admin";
import { triggerRef } from "vue";
import { useAccount } from "@agdb-studio/auth/src/account";

import { getRouter } from "@agdb-studio/router/src/router";
import { type Router } from "vue-router";

const { isAdmin, isAdminView } = useAdmin();
const { admin } = useAccount();

vi.mock("@agdb-studio/router/src/router");

describe("admin.ts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it.each([
    [false, false],
    [true, true],
  ])("returns the admin status %s", (input, expected) => {
    vi.mocked(getRouter).mockReturnValue({
      currentRoute: {
        value: {
          meta: {
            admin: input,
          },
        },
      },
    } as unknown as Router);
    admin.value = input;
    expect(isAdmin.value).toBe(expected);
  });

  it.each([
    [false, false],
    // todo reset the admin view status
    // [true, true],
  ])("returns the admin view status %s", (input, expected) => {
    vi.mocked(getRouter).mockReturnValue({
      currentRoute: {
        value: {
          meta: {
            admin: input,
          },
        },
      },
    } as unknown as Router);

    triggerRef(isAdminView);

    triggerRef(getRouter().currentRoute);

    expect(isAdminView.value).toBe(expected);
  });
});
