<script lang="ts" setup></script>

<template>
    <footer class="footer">
        <div class="page-container">
            <NavigationFooterLinks />
            <div class="copyright">Copyright @ 2024 agdb</div>
        </div>
    </footer>
</template>

<style lang="less" scoped>
.footer {
    padding-block: 1rem;
    box-shadow: 0 0 3px 0 var(--color-border);
    .copyright {
        font-size: 0.9rem;
        font-weight: 300;
    }
}
</style>
