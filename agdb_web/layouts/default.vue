<script lang="ts" setup></script>

<template>
    <div class="layout-default">
        <LayoutMainHeader />
        <LayoutMainContainer>
            <slot />
        </LayoutMainContainer>
        <LayoutMainFooter />
    </div>
</template>

<style lang="less" scoped>
.layout-default {
    width: 100%;
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: max-content minmax(max-content, 1fr) max-content;
    grid-template-areas:
        "header"
        "container"
        "footer";
    min-height: 100vh;
    grid-gap: 0;
}
#main-container {
    grid-area: container;
}
#main-header {
    grid-area: header;
}
#main-footer {
    grid-area: footer;
}
</style>
