import {
  createRouter as createRouterVue,
  type NavigationGuardNext,
  type RouteLocationNormalizedGeneric,
  type Router,
  type RouterOptions,
} from "vue-router";
import { useAuth } from "@agdb-studio/auth/src/auth";
import { useAccount } from "@agdb-studio/auth/src/account";

const { isLoggedIn } = useAuth();
const { admin, fetchUserStatus } = useAccount();

let router: Router | undefined = undefined;

export const beforeEach = async (
  to: RouteLocationNormalizedGeneric,
  from: RouteLocationNormalizedGeneric,
  next: NavigationGuardNext,
): Promise<void> => {
  await fetchUserStatus();

  if (!isLoggedIn.value) {
    if (to.name !== "login") {
      next({ name: "login", query: { redirect: to.fullPath } });
    } else {
      next();
    }
  } else if (to.meta.requiresAdmin) {
    console.debug(
      "Admin route detected:",
      to.fullPath,
      "Admin status:",
      admin.value,
    );
    if (admin.value) {
      next();
    } else {
      next({ name: "home" });
    }
  } else {
    if (to.name === "login") {
      next({ name: "home" });
    } else {
      next();
    }
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
