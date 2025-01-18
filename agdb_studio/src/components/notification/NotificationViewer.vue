<script lang="ts" setup>
import {
    notificationsReversed,
    newNotifications,
    viewerOpened,
    toggleViewerOpened,
    hasUnreadNotifications,
    clearNotifications,
    closeViewer,
} from "@/composables/notification/notificationStore";
import FadeTransition from "../transitions/FadeTransition.vue";
import NotificationItem from "./NotificationItem.vue";
import {
    CaNotification,
    ClCloseMd,
    CaNotificationNew,
    CaRowDelete,
} from "@kalimahapps/vue-icons";
</script>

<template>
    <div
        class="notification-wrapper"
        v-if="notificationsReversed.length"
        :class="{ hasNew: newNotifications.length }"
    >
        <FadeTransition>
            <div class="notification-viewer" v-if="viewerOpened">
                <div class="notification-header">
                    <h3><CaNotification /> Notifications</h3>

                    <div class="header-buttons">
                        <button
                            @click="
                                () => {
                                    clearNotifications(), toggleViewerOpened();
                                }
                            "
                            class="button button-transparent button-clear"
                            title="Clear all notifications"
                        >
                            <CaRowDelete />
                        </button>
                        <button
                            @click="closeViewer"
                            class="button button-transparent button-close"
                            title="Hide notifications"
                        >
                            <ClCloseMd />
                        </button>
                    </div>
                </div>
                <div class="notifications">
                    <TransitionGroup name="notification">
                        <NotificationItem
                            v-for="notification in notificationsReversed"
                            :key="notification.id"
                            :notification="notification"
                        />
                    </TransitionGroup>
                </div>
            </div>
        </FadeTransition>
        <div
            v-if="!viewerOpened && newNotifications.length"
            class="notification-flash"
        >
            <div class="notifications">
                <TransitionGroup name="notification">
                    <NotificationItem
                        v-for="notification in newNotifications"
                        :key="notification.id"
                        :notification="notification"
                    />
                </TransitionGroup>
            </div>
        </div>
        <button
            @click="toggleViewerOpened"
            class="button button-transparent notification-button"
            :class="{ shake: newNotifications.length }"
            :title="`${viewerOpened ? 'Hide' : 'Show'} notifications`"
        >
            <CaNotification v-if="!hasUnreadNotifications || viewerOpened" />
            <CaNotificationNew v-else />
        </button>
    </div>
</template>

<style lang="less" scoped>
.notification-wrapper {
    position: fixed;
    bottom: 0;
    right: 0;
    z-index: var(--z-index-notification);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    &.hasNew {
        z-index: var(--z-index-notification-new);
    }
}
.notifications {
    display: grid;
    grid-template-columns: 1fr;
    grid-gap: 0.5rem;
    width: 20rem;
    max-width: 100%;

    align-items: end;
    position: relative;
    --notifications-padding: 0;
    padding: var(--notifications-padding);
}
.notification-viewer {
    background-color: var(--color-background-soft);
    border: 1px solid var(--color-border);
    border-radius: 0.8rem;
    .notifications {
        overflow: auto;
        max-height: 30rem;
        min-height: 10rem;
        --notifications-padding: 0.5rem;
    }
}
.notification-flash {
    position: fixed;
    bottom: 3.8rem;
    right: 0.8rem;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    width: 20rem;
    max-width: 100%;
    overflow: visible;
    max-height: calc(100vh - 5rem);
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
    padding: 0.5rem;
    line-height: 0;
    font-size: 2rem;
    background-color: var(--color-background);
    border-radius: 50%;
    &.shake {
        animation: icon-shake 0.5s;
    }
    z-index: 100;
}
.notification {
    &-move,
    &-enter-active,
    &-leave-active {
        transition:
            opacity 0.3s ease-in-out,
            transform 0.3s ease-in-out;
    }
    &-enter-from,
    &-leave-to {
        opacity: 0;
        transform: translateY(-1rem);
    }
    &-leave-active {
        width: calc(100% - var(--notifications-padding) * 2);
        margin: 0 var(--notifications-padding);
        position: absolute;
    }

    .notification-flash &-leave-active {
        width: 100%;
        margin: 0;
        position: absolute;
    }
}
.notification-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.1rem 0;
    border-bottom: 1px solid var(--color-border);
    margin: 0;
    h3 {
        margin: 0;
        font-size: 1rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0rem 0.5rem;
    }
}
</style>
