import type { RouteRecordRaw } from "vue-router";
import { homeRedirect } from "@agdb-studio/dashboard/src/utils/homeRedirect";

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
          // Always redirects: to the home dashboard, or to the databases.
          beforeEnter: homeRedirect,
          component: () => import("@/views/DbView.vue"),
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
          path: "dashboards",
          name: "dashboards",
          component: () => import("@/views/DashboardsView.vue"),
        },
        {
          path: "dashboards/:owner/:name",
          name: "dashboard",
          component: () => import("@/views/DashboardView.vue"),
        },
        {
          path: "admin",
          meta: { requiresAdmin: true, admin: true },
          children: [
            {
              path: "",
              name: "admin",
              component: () => import("@/views/admin/AdminView.vue"),
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
            {
              path: "dashboards",
              name: "admin-dashboards",
              component: () => import("@/views/DashboardsView.vue"),
            },
            {
              path: "dashboards/:owner/:name",
              name: "admin-dashboard",
              component: () => import("@/views/DashboardView.vue"),
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
