<script lang="ts" setup>
const props = defineProps({
    name: {
        type: String,
        required: true,
    },
});
const { getLocalePath, localeCode } = useI18n();
const query = queryContent(localeCode.value, props.name);
</script>

<template>
    <div class="dropdown">
        <CommonLinkItem class="dropdown-link" :name="props.name" />
        <ContentNavigation v-slot="{ navigation }" :query="query">
            <div
                v-if="
                    (navigation.at(0)?.children?.at(0)?.children?.length ?? 0) >
                    1
                "
                class="dropdown-list"
            >
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
            </div>
        </ContentNavigation>
    </div>
</template>

<style lang="less" scoped>
.dropdown {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    .dropdown-link {
        margin-right: 1rem;
        font-size: 1.1rem;
    }
    .dropdown-list {
        display: none;
        position: absolute;
        top: 100%;
        z-index: 10;
        ul {
            list-style: none;
            margin-top: 1rem;
            background-color: var(--color-background);
            border: 1px solid var(--color-border);
            border-radius: 0.5rem;
            padding: 1.3rem 2rem;
            width: max-content;

            li {
                a {
                    font-weight: 300;
                    font-size: 1rem;
                    line-height: 2rem;
                }
            }
        }
    }
    &:hover,
    &:focus-within {
        .dropdown-list {
            display: block;
        }
    }
}
</style>
