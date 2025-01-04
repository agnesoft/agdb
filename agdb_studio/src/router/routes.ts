import MainLayout from "@/components/layouts/MainLayout.vue";

export const createRoutes = () => {
    return [
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
                    path: "/db",
                    name: "db",
                    component: () => import("@/views/DbView.vue"),
                },
            ],
        },
    ];
};
