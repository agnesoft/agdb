<script lang="ts" setup>
const { t, locale } = useI18n();
const localePath = useLocalePath();

const docsQuery = queryContent(locale.value, "docs");
const apiQuery = queryContent(locale.value, "api");
const enterpriseQuery = queryContent(locale.value, "enterprise");
const blogQuery = queryContent(locale.value, "blog");
</script>

<template>
    <header>
        <h3><NuxtLink :to="localePath('/docs')">Docs:</NuxtLink></h3>
        <nav>
            <ContentNavigation v-slot="{ navigation }" :query="docsQuery">
                <ul>
                    <template
                        v-for="link of navigation.at(0)?.children?.at(0)
                            ?.children"
                        :key="link._path"
                    >
                        <li v-if="link.children?.length">
                            <NuxtLink
                                v-if="link.children?.length"
                                :to="link._path"
                                >{{ link.navTitle || link.title }}</NuxtLink
                            >
                        </li>
                    </template>
                </ul>
            </ContentNavigation>
        </nav>

        <h3><NuxtLink :to="localePath('/api')">Api</NuxtLink></h3>
        <nav>
            <ContentNavigation v-slot="{ navigation }" :query="apiQuery">
                <ul>
                    <template
                        v-for="link of navigation.at(0)?.children?.at(0)
                            ?.children"
                        :key="link._path"
                    >
                        <li v-if="link.children?.length">
                            <NuxtLink
                                v-if="link.children?.length"
                                :to="link._path"
                                >{{ link.navTitle || link.title }}</NuxtLink
                            >
                        </li>
                    </template>
                </ul>
            </ContentNavigation>
        </nav>

        <h3><NuxtLink :to="localePath('/enterprise')">Enterprise</NuxtLink></h3>
        <nav>
            <ContentNavigation v-slot="{ navigation }" :query="enterpriseQuery">
                <ul>
                    <template
                        v-for="link of navigation.at(0)?.children?.at(0)
                            ?.children"
                        :key="link._path"
                    >
                        <li v-if="link.children?.length">
                            <NuxtLink
                                v-if="link.children?.length"
                                :to="link._path"
                                >{{ link.navTitle || link.title }}</NuxtLink
                            >
                        </li>
                    </template>
                </ul>
            </ContentNavigation>
        </nav>

        <h3><NuxtLink :to="localePath('/blog')">Blog</NuxtLink></h3>
        <nav>
            <ContentNavigation v-slot="{ navigation }" :query="blogQuery">
                <ul>
                    <template
                        v-for="link of navigation.at(0)?.children?.at(0)
                            ?.children"
                        :key="link._path"
                    >
                        <li v-if="link.children?.length">
                            <NuxtLink
                                v-if="link.children?.length"
                                :to="link._path"
                                >{{ link.navTitle || link.title }}</NuxtLink
                            >
                        </li>
                    </template>
                </ul>
            </ContentNavigation>
        </nav>

        <h3>Other:</h3>
        <ul>
            <li>
                <NuxtLink :to="localePath('/')">{{ t("menu.home") }}</NuxtLink>
            </li>
            <li>
                <NuxtLink :to="localePath(t('urls.about'))">{{
                    t("menu.about")
                }}</NuxtLink>
            </li>
            <li>
                <NuxtLink :to="localePath(t('urls.contact'))">{{
                    t("menu.contact")
                }}</NuxtLink>
            </li>
        </ul>
    </header>
    <img
        alt="agdb logo"
        class="logo"
        src="@/static/logo.svg"
        width="125"
        height="125"
    />
    <h1>agdb</h1>
</template>

<style lang="less" scoped></style>
