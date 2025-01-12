<script lang="ts" setup>
import {
    removeNotification,
    type AgdbNotification,
} from "@/composables/notification/notificationStore";
import type { PropType } from "vue";
import { ClCloseMd } from "@kalimahapps/vue-icons";

defineProps({
    notification: {
        type: Object as PropType<AgdbNotification>,
        required: true,
    },
});
</script>

<template>
    <div
        class="notification-item"
        :class="[
            notification.type,
            {
                new: notification.new,
                unread: !notification.read,
                withSeparator: notification.title && notification.message,
            },
        ]"
    >
        <h4 class="notification-title">{{ notification.title }}</h4>
        <button
            @click="removeNotification(notification.id)"
            class="button button-transparent button-close"
        >
            <ClCloseMd />
        </button>
        <div
            class="separator"
            v-if="notification.title && notification.message"
        ></div>
        <div class="notification-content" :class="notification.type">
            {{ notification.message }}
        </div>
        <div class="timestamp">
            {{ new Date(notification.timestamp).toLocaleString() }}
        </div>
    </div>
</template>

<style lang="less" scoped>
.notification-item {
    border-radius: 0.5rem;
    border: 1px solid transparent;
    --message-border-color: transparent;

    border-color: var(--message-border-color);
    overflow: hidden;
    background-color: var(--color-background-soft);
    transition:
        background-color 0.5s ease,
        outline 0.5s ease;

    &.info {
        --message-border-color: var(--info-color-2);
    }
    &.success {
        --message-border-color: var(--success-color-2);
    }
    &.warning {
        --message-border-color: var(--warning-color-2);
    }
    &.error {
        --message-border-color: var(--error-color-2);
    }

    &.unread {
        background-color: var(--color-background);
    }
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-rows: auto auto;
    grid-template-areas:
        "content close"
        "timestamp timestamp";
    &.withSeparator {
        grid-template-rows: auto 1px auto auto;
        grid-template-areas:
            "title close"
            "separator separator"
            "content content"
            "timestamp timestamp";
    }
}
.notification-title {
    grid-area: content;
    font-size: 1rem;
    font-weight: bold;
    padding: 0.2rem 0.5rem;
    color: var(--message-border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
    .withSeparator & {
        grid-area: title;
    }
}
.button-close {
    grid-area: close;
    color: var(--message-border-color);
    padding: 0.35rem 0.35rem 0 0;
    align-self: flex-start;
}
.separator {
    grid-area: separator;
    border-bottom: 1px solid var(--message-border-color);
}
.notification-content {
    grid-area: content;
    font-size: 0.9rem;
    padding: 0.5rem;
}
.timestamp {
    grid-area: timestamp;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    padding: 0.2rem 0.5rem;
    text-align: right;
}
</style>
