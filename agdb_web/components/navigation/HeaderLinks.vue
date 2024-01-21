<script lang="ts" setup></script>

<template>
    <nav class="header-links">
        <NavigationDropdownLinks :name="'docs'" />
        <NavigationDropdownLinks :name="'api'" />

        <NavigationDropdownLinks :name="'enterprise'" />

        <NavigationDropdownLinks :name="'blog'" />
    </nav>
</template>

<style lang="less" scoped>
.header-links {
    display: flex;
    align-items: center;
    a {
        margin-right: 1rem;
        font-size: 1.1rem;
    }
    @media screen and (max-width: @smallWidth) {
        flex-direction: column;
        align-items: flex-start;
        a {
            margin-right: 0;
            margin-bottom: 0.5rem;
        }
    }
}
</style>
