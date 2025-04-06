import { vi, describe, it, beforeEach, expect } from "vitest";
import { useAdmin } from "./admin";
import { triggerRef } from "vue";
import { useAccount } from "@/composables/profile/account";

import { getRouter } from "@agdb-studio/router/src/router";

const { isAdmin, isAdminView } = useAdmin();
const { admin } = useAccount();

describe("admin.ts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it.each([
    [false, false],
    [true, true],
  ])("returns the admin status %s", (input, expected) => {
    admin.value = input;
    expect(isAdmin.value).toBe(expected);
  });

  it.each([
    [false, false],
    [true, true],
  ])("returns the admin view status %s", (input, expected) => {
    const router = getRouter();
    router.currentRoute.value.meta.admin = input;
    triggerRef(router.currentRoute);
    expect(isAdminView.value).toBe(expected);
  });
});
