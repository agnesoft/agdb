import { mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h } from "vue";
import AppRoot from "./app.vue";

// Stubs for layout and UI components
const UApp = defineComponent({
  name: "UApp",
  setup(_, { slots }) {
    return () => h("div", slots.default?.());
  },
});
const UMain = defineComponent({
  name: "UMain",
  setup(_, { slots }) {
    return () => h("main", slots.default?.());
  },
});
const NuxtLoadingIndicator = defineComponent({
  name: "NuxtLoadingIndicator",
  setup: () => () => h("div"),
});
const AppHeader = defineComponent({
  name: "AppHeader",
  setup: () => () => h("header"),
});
const AppFooter = defineComponent({
  name: "AppFooter",
  setup: () => () => h("footer"),
});
const NuxtLayout = defineComponent({
  name: "NuxtLayout",
  setup(_, { slots }) {
    return () => h("section", slots.default?.());
  },
});
const NuxtPage = defineComponent({
  name: "NuxtPage",
  setup: () => () => h("article"),
});
const ClientOnly = defineComponent({
  name: "ClientOnly",
  setup(_, { slots }) {
    return () => h("div", slots.default?.());
  },
});
const LazyUContentSearch = defineComponent({
  name: "LazyUContentSearch",
  props: { files: { type: Object }, navigation: { type: Object } },
  setup(props) {
    return () =>
      h("div", {
        "data-files": props.files ? "yes" : "no",
        "data-navigation": props.navigation ? "yes" : "no",
      });
  },
});

// Mock app config and content queries used by app.vue
vi.mock("#imports", () => ({
  useAppConfig: () => ({ seo: { siteName: "agdb" } }),
  useAsyncData: async () => ({
    data: { value: [{ title: "Docs", path: "/docs" }] },
  }),
  useLazyAsyncData: async () => ({ data: { value: [{ section: "docs" }] } }),
  queryCollectionNavigation: () => ({}),
  queryCollectionSearchSections: () => ({}),
  useHead: () => {},
  useSeoMeta: () => {},
}));

describe("AppRoot (app.vue)", () => {
  it("renders header, footer, layout, page and search wiring", async () => {
    const wrapper = await mountSuspended(AppRoot, {
      global: {
        stubs: {
          UApp,
          UMain,
          NuxtLoadingIndicator,
          AppHeader,
          AppFooter,
          NuxtLayout,
          NuxtPage,
          ClientOnly,
          "client-only": ClientOnly,
          LazyUContentSearch,
          "lazy-u-content-search": LazyUContentSearch,
        },
      },
    });

    expect(wrapper.find("header").exists()).toBe(true);
    expect(wrapper.find("footer").exists()).toBe(true);
    expect(wrapper.find("main").exists()).toBe(true);
    expect(wrapper.find("section").exists()).toBe(true);
    expect(wrapper.find("article").exists()).toBe(true);
  });
});
