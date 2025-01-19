import MainLayout from "@/components/layouts/MainLayout.vue";
import type { RouteRecordRaw } from "vue-router";

// export const routes: RouteRecordRaw[] = [
//     {
//         path: "/login",
//         name: "login",
//         component: () => import("@/views/LoginView.vue"),
//     },
//     {
//         path: "",
//         component: MainLayout,
//         children: [
//             {
//                 path: "",
//                 name: "home",
//                 component: () => import("@/views/HomeView.vue"),
//             },
//             {
//                 path: "db",
//                 name: "db",
//                 component: () => import("@/views/DbView.vue"),
//             },
//             {
//                 path: "admin",
//                 meta: { requiresAdmin: true, admin: true },
//                 children: [
//                     {
//                         path: "",
//                         name: "admin",
//                         component: () => import("@/views/admin/AdminView.vue"),
//                     },
//                     {
//                         path: "users",
//                         name: "admin-users",
//                         component: () =>
//                             import("@/views/admin/AdminUserView.vue"),
//                     },
//                     {
//                         path: "db",
//                         name: "admin-db",
//                         component: () =>
//                             import("@/views/admin/AdminDbView.vue"),
//                     },
//                 ],
//             },
//         ],
//     },
// ];

export const createRoutes = (): RouteRecordRaw[] => {
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
                                import("@/views/admin/AdminUserView.vue"),
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
