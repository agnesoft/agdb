import "./assets/main.css";

import { createApp } from "vue";

import App from "./App.vue";
import { setOnUnauthorized } from "@agdb-studio/api/src/api";
import { createRouter, getRouter } from "@agdb-studio/router/src/router";
import { useAuth } from "@agdb-studio/auth/src/auth";
import { createWebHistory } from "vue-router";
import { createRoutes } from "./router/routes";
import { setupApiNotifications } from "./composables/apiNotifications";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: createRoutes(),
});

const app = createApp(App);

const { logout } = useAuth();

setOnUnauthorized(() => {
  try {
    logout(false, false);
    const r = getRouter();
    if (r.currentRoute.value.name !== "login") {
      r.push({ name: "login" }).catch(() => {
        // Navigation cancelled or duplicated — acceptable
      });
    }
  } catch {
    // Router not initialized yet (during initial client connection)
  }
});

setupApiNotifications();

app.use(router);

app.mount("#app");
