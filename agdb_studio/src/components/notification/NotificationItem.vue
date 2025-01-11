<script lang="ts" setup>
import type { AgdbNotification } from "@/composables/notification/notificationStore";
import type { PropType } from "vue";

defineProps({
    notification: {
        type: Object as PropType<AgdbNotification>,
        required: true,
    },
});
</script>

<template>
    <div class="notification-item" :class="notification.type">
        <h4 class="notification-title">{{ notification.title }}</h4>
        <div class="notification-content" :class="notification.type">
            {{ notification.message }}
        </div>
        <div class="timestamp">
            {{ new Date(notification.timestamp * 1000).toLocaleString() }}
        </div>
    </div>
</template>

<style lang="less" scoped>
.notification-item {
    border-radius: 0.5rem;
    border: 1px solid transparent;
    --message-background-color: var(--color-background);
    --message-border-color: transparent;

    border-color: var(--message-border-color);
    overflow: hidden;
    background-color: var(--color-background-soft);

    &.info {
        --message-background-color: var(--info-color-background);
        --message-border-color: var(--info-color);
    }
    &.success {
        --message-background-color: var(--success-color-background);
        --message-border-color: var(--success-color);
    }
    &.warning {
        --message-background-color: var(--warning-color-background);
        --message-border-color: var(--warning-color);
    }
    &.error {
        --message-background-color: var(--error-color-background);
        --message-border-color: var(--error-color);
    }

    .notification-title {
        font-size: 1rem;
        font-weight: bold;
        background-color: var(--message-background-color);
        padding: 0.2rem 0.5rem;
    }
    .notification-content {
        font-size: 0.9rem;
        padding: 0.5rem;
    }
    .timestamp {
        font-size: 0.75rem;
        color: var(--color-text-muted);
        padding: 0.2rem 0.5rem;
        text-align: right;
    }
}
</style>
