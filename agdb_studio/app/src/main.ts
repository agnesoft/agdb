import "./assets/main.css";

import { createApp } from "vue";

import App from "./App.vue";
import { createRouter } from "@agdb-studio/router/src/router";
import { createWebHistory } from "vue-router";
import { createRoutes } from "./router/routes";
import { setupApiNotifications } from "./composables/apiNotifications";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: createRoutes(),
});

const app = createApp(App);

setupApiNotifications();

app.use(router);

app.mount("#app");
