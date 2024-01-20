<script lang="ts" setup>
const {
    currentPagePath,
    setDefaultLocaleCode,
    localeExists,
    currentLocale,
    setLocaleCode,
    initI18n,
} = useI18n();
const route = useRoute();

initI18n();

if (
    !Array.isArray(route.params.slug) ||
    !route.params.slug.length ||
    !localeExists(route.params.slug[0])
) {
    setDefaultLocaleCode();
    currentPagePath.value = `${currentLocale.value?.code ?? "en"}${route.path}`;
} else {
    setLocaleCode(route.params.slug[0]);
    currentPagePath.value = `${route.path}`;
}
</script>
<template>
    <div>
        <NuxtLayout>
            <ContentDoc :path="currentPagePath ?? '/'" />
        </NuxtLayout>
    </div>
</template>
