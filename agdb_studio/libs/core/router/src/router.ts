import { useAccount } from "@agdb-studio/auth/src/account";
import { useAuth } from "@agdb-studio/auth/src/auth";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";
import {
  createRouter as createRouterVue,
  type RouteLocationNormalizedGeneric,
  type Router,
  type RouterOptions,
} from "vue-router";

const { isLoggedIn } = useAuth();
const { admin, fetchUserStatus } = useAccount();

let router: Router | undefined = undefined;

const logger = createLogger("Router");

export const beforeEach = async (
  to: RouteLocationNormalizedGeneric,
) => {
  if (to.name !== "login") {
    try {
      await fetchUserStatus();
    } catch {
      // 401s handled by the error interceptor
    }
  }

  if (!isLoggedIn.value) {
    if (to.name !== "login") {
      return { name: "login", query: { redirect: to.fullPath } };
    }
    return;
  }

  if (to.meta.requiresAdmin) {
    logger.debug(
      "Checking admin status for route:",
      to.fullPath,
      "Admin status:",
      admin.value,
    );
    if (!admin.value) {
      return { name: "home" };
    }
    return;
  }

  if (to.name === "login") {
    return { name: "home" };
  }
};

export const createRouter = (options: RouterOptions): Router => {
  router = createRouterVue(options);

  router.beforeEach(beforeEach);
  return router;
};

export const clearRouter = (): void => {
  if (router) {
    router = undefined;
  }
};

export const getRouter = (): Router => {
  if (!router) {
    throw new Error("Router not created yet. Call createRouter first.");
  }
  return router;
};
