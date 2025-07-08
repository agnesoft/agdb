import { describe, it, expect } from "vitest";
import { createRoutes } from "@/router/routes";
import type { RouteRecordRaw } from "vue-router";

import { vi } from "vitest";

export const get_token = vi.fn();
export const user_status = vi.fn();
export const db_list = vi.fn();
export const db_add = vi.fn();
export const db_backup = vi.fn().mockResolvedValue({});
export const db_restore = vi.fn().mockResolvedValue({});
export const db_clear = vi.fn().mockResolvedValue({});
export const db_convert = vi.fn().mockResolvedValue({});
export const db_remove = vi.fn().mockResolvedValue({});
export const db_delete = vi.fn().mockResolvedValue({});
export const db_optimize = vi.fn().mockResolvedValue({});
export const db_audit = vi.fn().mockResolvedValue({ data: [] });
export const db_copy = vi.fn().mockResolvedValue({});
export const db_rename = vi.fn().mockResolvedValue({});
export const db_user_list = vi.fn().mockResolvedValue({ data: [] });
export const db_user_add = vi.fn().mockResolvedValue({});
export const db_user_remove = vi.fn().mockResolvedValue({});
export const db_exec = vi.fn().mockResolvedValue({});
export const db_exec_mut = vi.fn().mockResolvedValue({});
export const user_change_password = vi.fn().mockResolvedValue(true);
export const admin_db_list = vi.fn();
export const admin_db_add = vi.fn();
export const admin_db_backup = vi.fn().mockResolvedValue({});
export const admin_db_restore = vi.fn().mockResolvedValue({});
export const admin_db_clear = vi.fn().mockResolvedValue({});
export const admin_db_convert = vi.fn().mockResolvedValue({});
export const admin_db_remove = vi.fn().mockResolvedValue({});
export const admin_db_delete = vi.fn().mockResolvedValue({});
export const admin_db_optimize = vi.fn().mockResolvedValue({});
export const admin_db_audit = vi.fn().mockResolvedValue({ data: [] });
export const admin_db_copy = vi.fn().mockResolvedValue({});
export const admin_db_rename = vi.fn().mockResolvedValue({});
export const admin_db_user_list = vi.fn().mockResolvedValue({ data: [] });
export const admin_db_user_add = vi.fn().mockResolvedValue({});
export const admin_db_user_remove = vi.fn().mockResolvedValue({});
export const admin_db_exec = vi.fn().mockResolvedValue({});
export const admin_db_exec_mut = vi.fn().mockResolvedValue({});
export const admin_user_list = vi.fn().mockResolvedValue({ data: [] });
export const admin_user_add = vi.fn().mockResolvedValue({});
export const admin_user_delete = vi.fn().mockResolvedValue({});
export const admin_user_logout = vi.fn().mockResolvedValue({});
export const cluster_admin_user_logout = vi.fn().mockResolvedValue({});
export const admin_user_change_password = vi.fn().mockResolvedValue(true);

export const client = vi.fn().mockResolvedValue({
  login: vi.fn().mockResolvedValue("token"),
  logout: vi.fn().mockResolvedValue(undefined),
  set_token: vi.fn(),
  get_token,
  reset_token: vi.fn(),
  interceptors: {
    request: {
      use: vi.fn(),
    },
    response: {
      use: vi.fn(),
    },
  },
  user_status,
  user_change_password,

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

  admin_user_list,
  admin_user_add,
  admin_user_delete,
  admin_user_logout,
  cluster_admin_user_logout,
  admin_user_change_password,
});
vi.mock("@agnesoft/agdb_api", () => {
  return {
    AgdbApi: {
      client,
    },
  };
});

const validateRoutes = (routes: RouteRecordRaw[]) => {
  routes.forEach((route) => {
    expect(route.path).toBeDefined();

    if (route.children) {
      validateRoutes(route.children);
    } else {
      expect(route.component).toBeDefined();
    }
  });
};

describe("routes", () => {
  it("creates routes", () => {
    const routes = createRoutes();
    expect(routes).toHaveLength(2);
    validateRoutes(routes);
  });
  it("all route components are async import functions", () => {
    const routes = createRoutes();

    const checkComponentIsAsyncImport = (route: RouteRecordRaw) => {
      if (route.component) {
        expect(typeof route.component).toBe("function");
        // Only call if it's a function (async import)
        const result = (route.component as () => Promise<unknown>)();
        expect(result).toBeInstanceOf(Promise);
      }
      if (route.children) {
        route.children.forEach(checkComponentIsAsyncImport);
      }
    };

    routes.forEach(checkComponentIsAsyncImport);
  });

  it("admin routes have correct meta and children", () => {
    const routes = createRoutes();
    const mainLayout = routes.find((r) => r.path === "");
    expect(mainLayout).toBeDefined();
    const adminRoute = mainLayout!.children!.find((r) => r.path === "admin");
    expect(adminRoute).toBeDefined();
    expect(adminRoute!.meta).toEqual({ requiresAdmin: true, admin: true });
    expect(adminRoute!.children).toHaveLength(3);

    const adminChildrenNames = adminRoute!.children!.map((r) => r.name);
    expect(adminChildrenNames).toEqual(["admin", "admin-users", "admin-db"]);
  });

  it("not-found route is present and matches wildcard", () => {
    const routes = createRoutes();
    const mainLayout = routes.find((r) => r.path === "");
    expect(mainLayout).toBeDefined();
    const notFoundRoute = mainLayout!.children!.find(
      (r) => r.name === "not-found",
    );
    expect(notFoundRoute).toBeDefined();
    expect(notFoundRoute!.path).toBe("/:pathMatch(.*)*");
  });
});
