import { computed } from "vue";
import { useAccount } from "./account";
import router from "@/router";

const { admin } = useAccount();

const isAdmin = computed(() => {
    return admin.value;
});

const isAdminView = computed(() => {
    return router.currentRoute.value.meta.admin;
});

export { isAdmin, isAdminView };
