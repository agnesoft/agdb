<script lang="ts" setup>
import { RouterLink, RouterView } from "vue-router";
import { useAccountStore } from "@/stores/account";
import LogoIcon from "@/components/base/icons/LogoIcon.vue";

const accountStore = useAccountStore();
</script>

<template>
    <div class="main-layout">
        <header>
            <LogoIcon />

            <div class="wrapper">
                <nav>
                    <RouterLink to="/">Home</RouterLink>
                    <RouterLink to="/about">About</RouterLink>
                </nav>
                <button
                    class="button button-warning logout-button"
                    @click="accountStore.logout"
                >
                    Logout
                </button>
            </div>
        </header>
        <main>
            <RouterView />
        </main>
        <footer></footer>
    </div>
</template>

<style lang="css" scoped>
.main-layout {
    min-height: 100dvh;
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: max-content 1fr max-content;
    grid-template-areas:
        "header"
        "main"
        "footer";
}

header {
    grid-area: header;
    line-height: 1.5;
}

main {
    grid-area: main;
    flex-grow: 1;
}

footer {
    grid-area: footer;
}

.logo-icon {
    --logo-icon-size: 100px;
    display: block;
    margin: 0 auto 2rem;
}

nav {
    font-size: 12px;
    text-align: center;
    margin-top: 2rem;
}

nav a.router-link-exact-active {
    color: var(--color-text);
}

nav a.router-link-exact-active:hover {
    background-color: transparent;
}

nav a {
    display: inline-block;
    padding: 0 1rem;
    border-left: 1px solid var(--color-border);
}

nav a:first-of-type {
    border: 0;
}

@media (min-width: 1024px) {
    header {
        display: flex;
        place-items: center;
        padding-right: calc(var(--section-gap) / 2);
    }

    .logo-icon {
        --logo-icon-size: 75px;
        margin: 0 2rem 0 1rem;
    }

    header .wrapper {
        display: flex;
        place-items: flex-start;
        flex-wrap: wrap;

        width: 100%;
        align-items: center;
        justify-content: space-between;
    }

    nav {
        text-align: left;
        margin-left: -1rem;
        font-size: 1rem;

        padding: 1rem 0;
        margin-top: 0;
    }
}
</style>
