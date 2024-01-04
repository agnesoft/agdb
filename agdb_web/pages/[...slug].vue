<script lang="ts" setup>
const { locale } = useI18n();
const route = useRoute();

const path =
    `/${locale.value}/` +
    (Array.isArray(route.params.slug)
        ? route.params.slug.join("/")
        : route.params.slug ?? "");
</script>
<template>
    <main>
        <img
            alt="agdb logo"
            class="logo"
            src="@/static/logo.svg"
            width="125"
            height="125"
        />
        <h1>agdb</h1>
        <ContentDoc :path="path" />
    </main>
</template>
