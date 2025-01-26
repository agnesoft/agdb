<script lang="ts" setup>
import AgdbMenu from "../base/menu/AgdbMenu.vue";
import { useUserActions } from "@/composables/profile/profileActions";

const { actions } = useUserActions();
</script>
<template>
    <div class="profile-dropdown-content">
        <AgdbMenu :actions="actions" />
    </div>
</template>
