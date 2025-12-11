<template>
  <div class="home-page">
    <h1>agdb</h1>
    <p class="subheading">Application native database for any use case.</p>
    <p class="subheading">
      no query language | performance independent of data size | 10X cost
      reduction
    </p>
    <div class="links">
      <UButton
        to="/docs/guides/quickstart"
        size="xl"
        trailing-icon="i-lucide-arrow-right"
      >
        Quickstart
      </UButton>
      <UButton to="/whyagdb" color="neutral" variant="outline" size="xl">
        Why agdb?
      </UButton>
      <UButton
        to="/enterprise/consultation"
        color="neutral"
        variant="outline"
        size="xl"
      >
        Expert consultation
      </UButton>
    </div>

    <p class="flex-row">
      <a href="/docs" class="border-image">
        <img width="100" src="/images/db.png" alt="db" />
      </a>
      <a href="/api-docs/openapi" class="border-image">
        <img width="100" src="/images/api.png" alt="api" />
      </a>
      <a href="docs/references/studio" class="border-image">
        <img width="100" src="/images/studio.png" alt="studio" />
      </a>
      <a href="docs/references/server" class="border-image">
        <img width="100" src="/images/server.png" alt="server" />
      </a>
      <a href="/enterprise" class="border-image">
        <img width="100" src="/images/cloud.png" alt="cloud" />
      </a>
    </p>
  </div>
</template>

<style scoped>
.home-page {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  min-height: 60vh;
}

h1 {
  font-size: 3rem;
  margin: 1rem;
  font-weight: 100;
}

.subheading {
  font-size: 1.3rem;
  font-weight: 100;
  margin-bottom: 3rem;
  opacity: 0.8;
}

.links {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: 1rem;
}

.flex-row {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  margin: 2rem 0;
  flex-wrap: wrap;
}

.border-image {
  filter: drop-shadow(1px 1px 1px rgb(0, 0, 0));
}
</style>
