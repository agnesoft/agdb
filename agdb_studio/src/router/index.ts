import { createRouter, createWebHistory } from "vue-router";
import { createRoutes } from "./routes";

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: createRoutes(),
});

router.beforeEach((to, from, next) => {
    if (to.name !== "login" && !localStorage.getItem("token")) {
        next({ name: "login" });
    } else {
        next();
    }
});

export default router;
