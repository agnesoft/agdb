// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    "@nuxt/eslint",
    "@nuxt/image",
    "@nuxt/ui",
    "@nuxt/content",
    "nuxt-og-image",
    "nuxt-llms",
  ],

  devtools: {
    enabled: true,
  },

  css: ["~/assets/css/main.css"],

  content: {
    build: {
      markdown: {
        toc: {
          searchDepth: 1,
        },
      },
    },
    // highlight: {
    //   theme: {
    //     default: "vs2015",
    //     light: "vs",
    //     dark: "vs",
    //   },
    //   langs: ["json", "javascript", "typescript", "bash", "shell"],
    // },
  },

  compatibilityDate: "2024-07-11",

  nitro: {
    prerender: {
      routes: ["/"],
      crawlLinks: true,
      autoSubfolderIndex: false,
    },
  },

  eslint: {
    config: {
      stylistic: {
        commaDangle: "never",
        braceStyle: "1tbs",
      },
    },
  },

  icon: {
    provider: "iconify",
  },

  llms: {
    domain: "https://agdb.agnesoft.com",
    title: "agdb - Application Native Database",
    description:
      "Application native database for any use case. No query language, performance independent of data size, 10X cost reduction.",
    full: {
      title: "agdb - Full Documentation",
      description:
        "Complete documentation for agdb - Application native database for any use case.",
    },
    sections: [
      {
        title: "Guides",
        contentCollection: "docs",
        contentFilters: [
          { field: "path", operator: "LIKE", value: "/docs/guides%" },
        ],
      },
      {
        title: "Examples",
        contentCollection: "docs",
        contentFilters: [
          { field: "path", operator: "LIKE", value: "/docs/examples%" },
        ],
      },
      {
        title: "References",
        contentCollection: "docs",
        contentFilters: [
          { field: "path", operator: "LIKE", value: "/docs/references%" },
        ],
      },
    ],
  },
});
