import { computed } from "vue";
import { useAccount } from "@agdb-studio/auth/src/account";
import { getRouter } from "./router";

const { admin } = useAccount();

const isAdmin = computed<boolean>(() => {
  return admin.value;
});

const isAdminView = computed<boolean>(() => {
  return !!getRouter().currentRoute.value.meta.admin;
});

export const useAdmin = () => {
  return { isAdmin, isAdminView };
};
