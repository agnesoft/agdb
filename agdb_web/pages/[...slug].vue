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
        <ContentDoc :path="path" />
    </main>
</template>
