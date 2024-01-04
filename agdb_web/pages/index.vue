<script lang="ts" setup>
const { t } = useI18n();
const localePath = useLocalePath();
</script>

<template>
    <NuxtLink :to="localePath('/')">{{ t("menu.home") }}</NuxtLink>
    <NuxtLink :to="localePath(t('urls.about'))">{{ t("menu.about") }}</NuxtLink>
    <NuxtLink :to="localePath(t('urls.contact'))">{{
        t("menu.contact")
    }}</NuxtLink>

    <h1>agdb</h1>
</template>

<style lang="less" scoped></style>
