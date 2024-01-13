<script lang="ts" setup>
const {
    t,
    localeCode,
    getLocalePath,
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

const docsQuery = queryContent(localeCode.value, "docs");
const apiQuery = queryContent(localeCode.value, "api");
const enterpriseQuery = queryContent(localeCode.value, "enterprise");
const blogQuery = queryContent(localeCode.value, "blog");
</script>
<template>
    <header>
        <h3><NuxtLink :to="getLocalePath('/docs')">Docs:</NuxtLink></h3>
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
                                :to="getLocalePath(link._path)"
                                >{{ link.navTitle || link.title }}</NuxtLink
                            >
                        </li>
                    </template>
                </ul>
            </ContentNavigation>
        </nav>
        <h3><NuxtLink :to="getLocalePath('/api')">Api</NuxtLink></h3>
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
                                :to="getLocalePath(link._path)"
                                >{{ link.navTitle || link.title }}</NuxtLink
                            >
                        </li>
                    </template>
                </ul>
            </ContentNavigation>
        </nav>

        <h3>
            <NuxtLink :to="getLocalePath('/enterprise')">Enterprise</NuxtLink>
        </h3>
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
                                :to="getLocalePath(link._path)"
                                >{{ link.navTitle || link.title }}</NuxtLink
                            >
                        </li>
                    </template>
                </ul>
            </ContentNavigation>
        </nav>

        <h3><NuxtLink :to="getLocalePath('/blog')">Blog</NuxtLink></h3>
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
                                :to="getLocalePath(link._path)"
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
        </ul>
    </header>
    <main>
        <ContentDoc :path="currentPagePath ?? '/'" />
    </main>
</template>
