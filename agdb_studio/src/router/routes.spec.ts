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
});
