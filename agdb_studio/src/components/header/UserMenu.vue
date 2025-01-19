<script lang="ts" setup>
import AgdbMenu from "../base/menu/AgdbMenu.vue";
import { useUserActions } from "@/composables/user/userActions";

const { actions } = useUserActions();
</script>
<template>
    <div class="user-dropdown-content">
        <AgdbMenu :actions="actions" />
    </div>
</template>
