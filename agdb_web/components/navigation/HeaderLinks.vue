<script lang="ts" setup>
// const { t, getLocalePath } = useI18n();
// const docsQuery = queryContent(localeCode.value, "docs");
// const apiQuery = queryContent(localeCode.value, "api");
// const enterpriseQuery = queryContent(localeCode.value, "enterprise");
// const blogQuery = queryContent(localeCode.value, "blog");
</script>

<template>
    <nav class="header-links">
        <CommonLinkItem :name="'docs'" />
        <!-- <ContentNavigation v-slot="{ navigation }" :query="docsQuery">
            <ul>
                <template
                    v-for="link of navigation.at(0)?.children?.at(0)?.children"
                    :key="link._path"
                >
                    <li v-if="link.children?.length">
                        <NuxtLink
                            v-if="link.children?.length"
                            :to="getLocalePath(link._path)"
                            >{{ link.navTitle || link.title }}</NuxtLink
                        >
                    </li>
                </template>
            </ul>
        </ContentNavigation> -->
        <CommonLinkItem :name="'api'" />
        <!-- <ContentNavigation v-slot="{ navigation }" :query="apiQuery">
            <ul>
                <template
                    v-for="link of navigation.at(0)?.children?.at(0)?.children"
                    :key="link._path"
                >
                    <li v-if="link.children?.length">
                        <NuxtLink
                            v-if="link.children?.length"
                            :to="getLocalePath(link._path)"
                            >{{ link.navTitle || link.title }}</NuxtLink
                        >
                    </li>
                </template>
            </ul>
        </ContentNavigation> -->

        <CommonLinkItem :name="'enterprise'" />

        <!-- <ContentNavigation v-slot="{ navigation }" :query="enterpriseQuery">
            <ul>
                <template
                    v-for="link of navigation.at(0)?.children?.at(0)?.children"
                    :key="link._path"
                >
                    <li v-if="link.children?.length">
                        <NuxtLink
                            v-if="link.children?.length"
                            :to="getLocalePath(link._path)"
                            >{{ link.navTitle || link.title }}</NuxtLink
                        >
                    </li>
                </template>
            </ul>
        </ContentNavigation> -->

        <CommonLinkItem :name="'blog'" />
        <!-- <ContentNavigation v-slot="{ navigation }" :query="blogQuery">
            <ul>
                <template
                    v-for="link of navigation.at(0)?.children?.at(0)?.children"
                    :key="link._path"
                >
                    <li v-if="link.children?.length">
                        <NuxtLink
                            v-if="link.children?.length"
                            :to="getLocalePath(link._path)"
                            >{{ link.navTitle || link.title }}</NuxtLink
                        >
                    </li>
                </template>
            </ul>
        </ContentNavigation> -->

        <!-- <h3>Other:</h3>
        <ul>
            <li>
                <NuxtLink :to="getLocalePath('/')">
                    {{ t("menu.home") }}
                </NuxtLink>
            </li>
            <li>
                <NuxtLink :to="getLocalePath(t('url.about'))">
                    {{ t("menu.about") }}
                </NuxtLink>
            </li>
            <li>
                <NuxtLink :to="getLocalePath(t('url.contact'))">
                    {{ t("menu.contact") }}
                </NuxtLink>
            </li>
        </ul> -->
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
