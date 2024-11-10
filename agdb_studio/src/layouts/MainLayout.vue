<script lang="ts" setup>
import { RouterLink, RouterView } from "vue-router";
import { useAuth } from "@/composables/user/auth";
import LogoIcon from "@/components/base/icons/LogoIcon.vue";
import { useAccount } from "@/composables/user/account";

const { logout } = useAuth();
const { username } = useAccount();
</script>

<template>
    <div class="main-layout">
        <header>
            <RouterLink to="/"><LogoIcon /></RouterLink>

            <div class="wrapper">
                <nav>
                    <RouterLink to="/">Home</RouterLink>
                    <RouterLink to="/db">Databases</RouterLink>
                </nav>
                <button
                    class="button button-warning logout-button"
                    @click="logout"
                >
                    Logout {{ username }}
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
    overflow: hidden;
}

header {
    grid-area: header;
    line-height: 1.5;
}

main {
    grid-area: main;
    flex-grow: 1;
    padding: 1rem;
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

nav a:not(.router-link-exact-active) {
    color: var(--color-text);
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
