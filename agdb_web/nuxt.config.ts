// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    "@nuxt/eslint",
    "@nuxt/image",
    "@nuxt/ui",
    "@nuxt/content",
    "nuxt-og-image",
    "nuxt-llms",
    "@nuxt/test-utils/module",
  ],
  ssr: true,

  devtools: {
    enabled: true,
  },

  css: ["~/assets/css/main.css"],

  content: {
    experimental: {
      nativeSqlite: true,
    },
    build: {
      markdown: {
        toc: {
          searchDepth: 3,
        },
        highlight: {
          theme: {
            default: "github-light", // or "vs", "slack-light", etc.
            dark: "github-dark", // or "tokyo-night", "dracula", etc.
          },
          langs: [
            "json",
            "js",
            "ts",
            "html",
            "css",
            "vue",
            "shell",
            "mdc",
            "md",
            "yaml",
            "rs",
            "bash",
            "php",
            "python",
          ],
        },
      },
    },
  },

  compatibilityDate: "2024-07-11",

  nitro: {
    // prerender: {
    //   routes: ["/"],
    //   crawlLinks: true,
    //   autoSubfolderIndex: false,
    // },
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
    title: "agdb Documentation",
    description: "Guides, API docs, references, and examples for agdb.",
    full: {
      title: "agdb Full Documentation",
      description:
        "Complete docs for using agdb, running agdb_server, and integrating API clients.",
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
      {
        title: "API Clients",
        contentCollection: "docs",
        contentFilters: [
          { field: "path", operator: "LIKE", value: "/api-docs%" },
        ],
      },
      {
        title: "Blog",
        contentCollection: "docs",
        contentFilters: [{ field: "path", operator: "LIKE", value: "/blog%" }],
      },
      {
        title: "Enterprise",
        contentCollection: "docs",
        contentFilters: [
          { field: "path", operator: "LIKE", value: "/enterprise%" },
        ],
      },
    ],
  },
});
