export default defineAppConfig({
  ui: {
    colors: {
      primary: "green",
      neutral: "slate",
    },
    footer: {
      slots: {
        root: "border-t border-default",
        left: "text-sm text-muted",
      },
    },
  },
  seo: {
    siteName: "agdb - Application Native Database",
  },
  header: {
    title: "",
    to: "/",
    logo: {
      alt: "",
      light: "",
      dark: "",
    },
    search: true,
    colorMode: true,
    internalLinks: [
      { label: "API", to: "/api-docs/openapi" },
      { label: "Docs", to: "/docs/guides/quickstart" },
      { label: "Enterprise", to: "/enterprise" },
      { label: "Blog", to: "/blog" },
    ],
    links: [
      {
        icon: "i-simple-icons-github",
        to: "https://github.com/agnesoft/agdb",
        target: "_blank",
        "aria-label": "GitHub",
      },
    ],
  },
  footer: {
    credits: `Copyright © ${new Date().getFullYear()} agdb`,
    colorMode: false,
    links: [
      {
        icon: "i-simple-icons-github",
        to: "https://github.com/agnesoft/agdb",
        target: "_blank",
        "aria-label": "agdb on GitHub",
      },
    ],
  },
  toc: {
    title: "Table of Contents",
    bottom: {
      title: "Community",
      edit: "https://github.com/agnesoft/agdb/edit/main/agdb_web_nuxt/content",
      links: [
        {
          icon: "i-lucide-star",
          label: "Star on GitHub",
          to: "https://github.com/agnesoft/agdb",
          target: "_blank",
        },
        {
          icon: "i-lucide-book-open",
          label: "API Reference",
          to: "/docs/references/rust",
          target: "_blank",
        },
      ],
    },
  },
});
