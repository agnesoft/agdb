import { describe, it, expect } from "vitest";
import { createRoutes } from "@/router/routes";

const validateRoutes = (routes: ReturnType<typeof createRoutes>) => {
    routes.forEach((route) => {
        expect(route.path).toBeDefined();
        expect(route.component).toBeDefined();
        if (route.children) {
            validateRoutes(route.children);
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
