import { describe, it, expect } from "vitest";
import { createRoutes } from "@/router/routes";
import type { RouteRecordRaw } from "vue-router";

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
