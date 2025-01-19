import { computed } from "vue";
import { useAccount } from "./account";
import router from "@/router";

const { admin } = useAccount();

const isAdmin = computed<boolean>(() => {
    console.log("isAdmin", admin.value);
    return admin.value;
});

const isAdminView = computed<boolean>(() => {
    return !!router.currentRoute.value.meta.admin;
});

export const useAdmin = () => {
    return { isAdmin, isAdminView };
};
