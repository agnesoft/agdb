<script lang="ts" setup></script>

<template>
    <main class="main-container">
        <div class="page-container">
            <slot />
        </div>
    </main>
</template>

<style lang="less" scoped>
.main-container {
    grid-area: container;
    padding-block: 1rem;
    padding-inline: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;

    ::v-deep(h1, h2, h3, h4, h5, h6) {
        font-size: 2rem;
        font-weight: 300;
        color: var(--color-heading);
        margin-block: 1rem;
    }
    ::v-deep(p) {
        font-size: 1.1rem;
        font-weight: 300;
        color: var(--color-text);
        margin-block: 1rem;
    }
}
</style>
