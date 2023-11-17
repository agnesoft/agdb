import { createRouter, createWebHistory } from "vue-router";
import MainLayout from "@/layouts/MainLayout.vue";

const routes = [
    {
        path: "/login",
        name: "login",
        component: () => import("@/views/LoginView.vue"),
    },
    {
        path: "",
        component: MainLayout,
        children: [
            {
                path: "",
                name: "home",
                component: () => import("@/views/HomeView.vue"),
            },
            {
                path: "/about",
                name: "about",
                component: () => import("@/views/AboutView.vue"),
            },
        ],
    },
];

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes,
});

export { routes };

export default router;
