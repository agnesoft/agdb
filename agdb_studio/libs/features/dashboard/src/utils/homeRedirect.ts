import type { RouteLocationRaw } from "vue-router";
import { useDashboardStore } from "../composables/dashboardStore";

/**
 * Where the start page goes: the user's home dashboard if one is set,
 * otherwise (or if it cannot be fetched) the database list.
 */
export const homeRedirect = async (): Promise<RouteLocationRaw> => {
  try {
    const home = await useDashboardStore().fetchHome();
    if (home) {
      return {
        name: "dashboard",
        params: { owner: home.owner, name: home.name },
      };
    }
  } catch {
    // The API error interceptor shows a notification.
  }
  return { name: "db" };
};
