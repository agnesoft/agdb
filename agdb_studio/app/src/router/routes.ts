import type { RouteRecordRaw } from "vue-router";

export const createRoutes = (): RouteRecordRaw[] => {
  return [
    {
      path: "/login",
      name: "login",
      component: () => import("@/views/LoginView.vue"),
    },
    {
      path: "",
      component: () => import("@/components/layouts/MainLayout.vue"),
      children: [
        {
          path: "",
          name: "home",
          // component: () => import("@/views/HomeView.vue"),
          redirect: { name: "db" },
        },
        {
          path: "db",
          name: "db",
          component: () => import("@/views/DbView.vue"),
        },
        {
          path: "query/:owner/:db",
          name: "query",
          component: () => import("@/views/QueryView.vue"),
        },
        {
          path: "admin",
          meta: { requiresAdmin: true, admin: true },
          children: [
            {
              path: "",
              name: "admin",
              // component: () => import("@/views/admin/AdminView.vue"),
              redirect: { name: "admin-db" },
            },
            {
              path: "users",
              name: "admin-users",
              component: () => import("@/views/admin/AdminUserView.vue"),
            },
            {
              path: "db",
              name: "admin-db",
              component: () => import("@/views/admin/AdminDbView.vue"),
            },
            {
              path: "query/:owner/:db",
              name: "admin-query",
              component: () => import("@/views/QueryView.vue"),
            },
          ],
        },
        {
          path: "/:pathMatch(.*)*",
          name: "not-found",
          component: () => import("@/views/NotFoundView.vue"),
        },
      ],
    },
  ];
};
