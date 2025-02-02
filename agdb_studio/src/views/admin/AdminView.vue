<script lang="ts" setup>
import { RouterLink } from "vue-router";</script>

<template>
    <div class="admin-view">
        <h1>Admin View</h1>

        <p>Hi there! This is the admin view. Nothing to see here yet.</p>

        <p>
            <RouterLink to="/admin/db">Go to the database list</RouterLink>
        </p>
    </div>
</template>

<style lang="less" scoped>
.admin-view {
    padding: 20px;
    text-align: center;
    h1 {
        margin-bottom: 20px;
    }
    p {
        margin-bottom: 10px;
    }
}
</style>
