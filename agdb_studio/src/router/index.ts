import { createRouter, createWebHistory } from "vue-router";
import { createRoutes } from "./routes";
import { isLoggedIn, logout } from "@/services/auth.service";

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: createRoutes(),
});

router.beforeEach((to, from, next) => {
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
});

export default router;
