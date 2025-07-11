<script lang="ts" setup>
import AgdbMenu from "@agdb-studio/common/src/components/menu/AgdbMenu.vue";
import { useUserActions } from "../composables/profileActions";

const { actions } = useUserActions();
</script>
<template>
  <div class="profile-dropdown-content">
    <AgdbMenu :actions="actions" />
  </div>
</template>
