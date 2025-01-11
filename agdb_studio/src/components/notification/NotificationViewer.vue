<script lang="ts" setup>
import { useNotificationStore } from "@/composables/notification/notificationStore";
import { ref } from "vue";
import FadeTransition from "../transitions/FadeTransition.vue";
import NotificationItem from "./NotificationItem.vue";
import { CaNotification } from "@kalimahapps/vue-icons";

const { notifications, newNotifications } = useNotificationStore();
const opened = ref(false);
const toggleOpened = (): void => {
    opened.value = !opened.value;
};
</script>

<template>
    <div class="notification-wrapper" v-if="notifications.length">
        <FadeTransition>
            <div class="notification-viewer" v-if="opened">
                <TransitionGroup name="notification">
                    <NotificationItem
                        v-for="notification in notifications"
                        :key="notification.id"
                        :notification="notification"
                    />
                </TransitionGroup>
            </div>
        </FadeTransition>
        <div
            v-if="!opened && newNotifications.length"
            class="notification-flash"
        >
            <TransitionGroup name="notification">
                <NotificationItem
                    v-for="notification in newNotifications"
                    :key="notification.id"
                    :notification="notification"
                />
            </TransitionGroup>
        </div>
        <button
            @click="toggleOpened"
            class="button button-transparent notification-button"
            :class="{ shake: newNotifications.length }"
            :title="`${opened ? 'Hide' : 'Show'} notifications`"
        >
            <CaNotification />
        </button>
    </div>
</template>

<style lang="less" scoped>
.notification-wrapper {
    position: fixed;
    bottom: 0;
    right: 0;
    z-index: 10;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
}
.notification-viewer,
.notification-flash {
    display: grid;
    grid-template-columns: 1fr;
    grid-gap: 0.5rem;
    width: 20rem;
    max-width: 100%;
    overflow: auto;
    border-radius: 0.8rem;
}
.notification-viewer {
    padding: 0.5rem;
    background-color: var(--color-background-soft);
    border: 1px solid var(--color-border);
    max-height: 30rem;
    min-height: 10rem;
}
.notification-flash {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    width: 20rem;
    max-width: 100%;
    overflow: auto;
}
@keyframes icon-shake {
    0% {
        transform: rotate(0);
    }
    25% {
        transform: rotate(10deg);
    }
    50% {
        transform: rotate(-10deg);
    }
    75% {
        transform: rotate(5deg);
    }
    100% {
        transform: rotate(0);
    }
}
.notification-button {
    padding: 0.5rem 0.5rem 0 0;
    // padding: 0;
    font-size: 2rem;
    &.shake {
        animation: icon-shake 0.5s;
    }
}
.notification {
    &-enter-active,
    &-leave-active {
        transition:
            opacity 0.3s,
            transform 0.3s;
    }
    &-enter-from,
    &-leave-to {
        opacity: 0;
        transform: translateY(1rem);
    }
}
</style>
