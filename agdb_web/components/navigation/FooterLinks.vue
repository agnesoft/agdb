<script lang="ts" setup></script>

<template>
    <div class="footer-links">
        <div class="footer-column">
            <CommonLinkItem name="docs" />
            <CommonLinkItem name="api" />
            <CommonLinkItem name="enterprise" />
        </div>
        <div class="footer-column">
            <CommonLinkItem name="blog" />
            <CommonLinkItem name="about" />
            <CommonLinkItem name="contact" />
        </div>
        <div class="footer-column">
            <CommonLinkItem name="privacy-policy" />
            <CommonLinkItem name="terms-and-conditions" />
        </div>
    </div>
</template>

<style lang="less" scoped>
.footer-links {
    display: flex;
    flex-direction: row;
    justify-content: space-evenly;
    align-items: center;
    margin: 0 0 1rem 0;
    .footer-column {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        margin: 0 1rem;
    }
}
</style>
