<script lang="ts" setup>
const i18n = useI18n();
</script>

<template>
    <header class="header">
        <div class="page-container">
            <NuxtLink class="logo-wrapper" :to="i18n.getLocalePath('/')">
                <img
                    src="@/static/logo.svg"
                    alt="agdb logo"
                    class="logo"
                    width="80"
                    height="80"
                />
                <span>agdb</span>
            </NuxtLink>
            <NavigationHeaderLinks />
        </div>
    </header>
</template>

<style lang="less" scoped>
.header {
    display: flex;
    align-items: center;
    justify-content: space-between;

    // border-bottom: 1px solid var(--color-border);
    top: 0;
    left: 0;
    right: 0;
    box-shadow: 0 0 3px 0 var(--color-border);
    .page-container {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    .logo-wrapper {
        display: flex;
        align-items: center;
        text-decoration: none;
        font-size: 1.5rem;
        font-weight: 700;
        img {
            margin-right: 0.5rem;
            width: 70px;
            height: 70px;
        }
    }
}
</style>
