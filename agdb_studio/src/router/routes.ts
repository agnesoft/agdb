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
                    path: "db",
                    name: "db",
                    component: () => import("@/views/DbView.vue"),
                },
                {
                    path: "admin",
                    meta: { requiresAdmin: true, admin: true },
                    children: [
                        {
                            path: "",
                            name: "admin",
                            component: () =>
                                import("@/views/admin/AdminView.vue"),
                        },
                        {
                            path: "users",
                            name: "admin-users",
                            component: () =>
                                import("@/views/admin/AdminUsersView.vue"),
                        },
                        {
                            path: "db",
                            name: "admin-db",
                            component: () =>
                                import("@/views/admin/AdminDbView.vue"),
                        },
                    ],
                },
            ],
        },
    ];
};
