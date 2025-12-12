import { mountSuspended, mockNuxtImport } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h, ref } from "vue";
import Page from "./[...slug].vue";

// Stubs for page layout components
const UPage = defineComponent({
  name: "UPage",
  setup(_, { slots }) {
    return () => h("div", [slots.right?.(), slots.default?.()]);
  },
});
const UPageHeader = defineComponent({
  name: "UPageHeader",
  props: ["title", "description", "headline"],
  setup(_, { slots }) {
    return () => h("header", [slots.links?.(), slots.default?.()]);
  },
});
const UPageBody = defineComponent({
  name: "UPageBody",
  setup(_, { slots }) {
    return () => h("main", slots.default?.());
  },
});
const UContentSurround = defineComponent({
  name: "UContentSurround",
  props: ["surround"],
  setup() {
    return () => h("nav");
  },
});
const UContentToc = defineComponent({
  name: "UContentToc",
  props: ["title", "links"],
  setup(_, { slots }) {
    return () => h("aside", [slots.bottom?.()]);
  },
});
const UPageLinks = defineComponent({
  name: "UPageLinks",
  props: ["title", "links"],
  setup() {
    return () => h("div");
  },
});
const USeparator = defineComponent({
  name: "USeparator",
  props: ["type"],
  setup() {
    return () => h("hr");
  },
});
const UButton = defineComponent({
  name: "UButton",
  props: ["label"],
  setup(props) {
    return () => h("button", props.label);
  },
});
const PageHeaderLinks = defineComponent({
  name: "PageHeaderLinks",
  setup() {
    return () => h("div");
  },
});
const ContentRenderer = defineComponent({
  name: "ContentRenderer",
  props: { value: { type: Object } },
  setup(props) {
    return () => h("article", { "data-title": props.value?.title ?? "" });
  },
});

// Hoisted mocks
mockNuxtImport("useRoute", () => () => ({
  path: "/docs/guides/getting-started",
}));
mockNuxtImport("useAppConfig", () => () => ({
  toc: {
    title: "On this page",
    bottom: {
      title: "More",
      edit: "https://github.com/agnesoft/agdb/edit/main/docs",
      links: [
        { icon: "i-lucide-link", label: "Extra", to: "https://example.com" },
      ],
    },
  },
}));
mockNuxtImport("useSeoMeta", () => () => {});
// Mock auto-imports module to provide OG image composable
vi.mock("#imports", () => ({
  defineOgImageComponent: () => ({}),
}));
// Fallback: stub global in case auto-import resolves to global function
vi.stubGlobal("defineOgImageComponent", () => ({}));

// Mock findPageHeadline to return a stable string
vi.mock("@nuxt/content/utils", () => ({ findPageHeadline: () => "Headline" }));

// useAsyncData is called twice with different keys
mockNuxtImport("useAsyncData", () => (key: string, _fn: unknown) => {
  if (key.endsWith("-surround")) {
    return Promise.resolve({
      data: { value: [{ path: "/docs/prev" }, { path: "/docs/next" }] },
    });
  }
  return Promise.resolve({
    data: {
      value: {
        path: "/docs/guides/getting-started",
        stem: "getting-started",
        extension: "md",
        title: "Getting Started",
        description: "Intro",
        body: { toc: { links: [{ id: "a" }] } },
      },
    },
  });
});

describe("Dynamic docs page ([...slug])", () => {
  it("renders content, toc, and links", async () => {
    const wrapper = await mountSuspended(Page, {
      global: {
        provide: { navigation: ref([{ path: "/docs", children: [] }]) },
        stubs: {
          UPage,
          UPageHeader,
          UPageBody,
          UContentSurround,
          UContentToc,
          UPageLinks,
          USeparator,
          UButton,
          PageHeaderLinks,
          ContentRenderer,
        },
      },
    });
    // allow reactive updates
    const { nextTick } = await import("vue");
    await nextTick();
    await nextTick();

    const article = wrapper.find("article[data-title]");
    expect(article.exists()).toBe(true);
  });
});
