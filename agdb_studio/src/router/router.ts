import {
    createRouter,
    createWebHistory,
    type NavigationGuardNext,
    type RouteLocationNormalizedGeneric,
} from "vue-router";
import { createRoutes } from "./routes";
import { useAuth } from "@/composables/user/auth";
import { useAccount } from "@/composables/user/account";

const { isLoggedIn } = useAuth();
const { admin, fetchUserStatus } = useAccount();

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: createRoutes(),
});

export const beforeEach = async (
    to: RouteLocationNormalizedGeneric,
    from: RouteLocationNormalizedGeneric,
    next: NavigationGuardNext,
) => {
    await fetchUserStatus();

    if (!isLoggedIn.value) {
        if (to.name !== "login") {
            next({ name: "login", query: { redirect: to.fullPath } });
        } else {
            next();
        }
    } else if (to.meta.requiresAdmin) {
        if (admin.value) {
            next();
        } else {
            // todo redirect to 404
            next({ name: "home" });
        }
    } else {
        if (to.name === "login") {
            next({ name: "home" });
        } else {
            next();
        }
    }
};

router.beforeEach(beforeEach);

export default router;
