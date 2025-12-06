<script lang="ts" setup>
import { RouterLink, RouterView } from "vue-router";
import LogoIcon from "@agdb-studio/design/src/components/icons/LogoIcon.vue";
import AgdbModal from "@agdb-studio/common/src/components/modal/AgdbModal.vue";
import FadeTransition from "@agdb-studio/design/src/components/transitions/FadeTransition.vue";
import NotificationViewer from "@agdb-studio/notification/src/components/NotificationViewer.vue";
import ProfileDropdown from "@agdb-studio/profile/src/components/ProfileDropdown.vue";
import ClusterStatusIndicator from "@agdb-studio/cluster/src/components/ClusterStatusIndicator.vue";
import { computed } from "vue";
import { useAdmin } from "@agdb-studio/profile/src/composables/admin";

const { isAdminView } = useAdmin();

const homeLink = computed(() => (isAdminView.value ? "/admin" : "/"));

const links = computed(() => {
  if (isAdminView.value) {
    return [
      { to: "/admin/db", text: "Databases" },
      { to: "/admin/users", text: "Users" },
    ];
  }
  return [{ to: "/db", text: "Databases" }];
});
</script>

<template>
  <div class="main-layout">
    <header>
      <RouterLink :to="homeLink" class="logo-wrapper">
        <LogoIcon />
        <span v-if="isAdminView" class="admin-label"> admin </span>
      </RouterLink>

      <div class="wrapper">
        <nav>
          <RouterLink v-for="link of links" :key="link.to" :to="link.to">{{
            link.text
          }}</RouterLink>
        </nav>
        <div class="header-actions">
          <ClusterStatusIndicator />
          <ProfileDropdown />
        </div>
      </div>
    </header>
    <main>
      <RouterView />
    </main>
    <footer></footer>
    <NotificationViewer />
    <FadeTransition>
      <AgdbModal />
    </FadeTransition>
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
  max-width: 100vw;
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

.logo-wrapper {
  position: relative;
}

.admin-label {
  font-size: 0.8rem;
  background-color: var(--red-2);
  color: var(--white);
  padding: 0.1rem 0.5rem;
  border-radius: 5px;
  position: absolute;
  bottom: 0;
  right: 0.8rem;
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
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
