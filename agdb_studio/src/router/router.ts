import {
    createRouter,
    createWebHistory,
    type NavigationGuardNext,
    type RouteLocationNormalizedGeneric,
} from "vue-router";
import { createRoutes } from "./routes";
import { isLoggedIn, logout } from "@/services/auth.service";

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: createRoutes(),
});

export const beforeEach = (
    to: RouteLocationNormalizedGeneric,
    from: RouteLocationNormalizedGeneric,
    next: NavigationGuardNext,
) => {
    if (isLoggedIn()) {
        if (to.name === "login") {
            logout();
        }
        next();
    } else {
        if (to.name !== "login") {
            next({ name: "login" });
        } else {
            next();
        }
    }
};

router.beforeEach(beforeEach);

export default router;
