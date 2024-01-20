<script lang="ts" setup>
const props = defineProps({
    name: {
        type: String,
        required: true,
    },
});

const i18n = useI18n();
const linkPath = computed(() =>
    i18n.getLocalePath(i18n.t("url." + props.name)),
);
const title = computed(() => i18n.t("menu." + props.name));
</script>

<template>
    <NuxtLink class="link-item" :to="linkPath" :title="title">
        <span>{{ title }}</span>
    </NuxtLink>
</template>

<style lang="less" scoped></style>
