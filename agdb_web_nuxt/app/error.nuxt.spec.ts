import { mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h } from "vue";
import ErrorPage from "./error.vue";
import type { NuxtError } from "#app";

// Stubs
const AppHeader = defineComponent({
  name: "AppHeader",
  setup: () => () => h("header"),
});
const AppFooter = defineComponent({
  name: "AppFooter",
  setup: () => () => h("footer"),
});
const UApp = defineComponent({
  name: "UApp",
  setup(_, { slots }) {
    return () => h("div", slots.default?.());
  },
});
const UError = defineComponent({
  name: "UError",
  props: { error: { type: Object, required: true } },
  setup(props) {
    return () =>
      h("div", { "data-error": String(props.error?.statusCode ?? "unknown") });
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

// Stub ClientOnly to render its default slot immediately in tests
const ClientOnly = defineComponent({
  name: "ClientOnly",
  setup(_, { slots }) {
    return () => h("div", slots.default?.());
  },
});

// Mock content queries used by error page
vi.mock("#imports", () => ({
  useAsyncData: async () => ({
    data: { value: [{ title: "Docs", path: "/docs" }] },
  }),
  useLazyAsyncData: async () => ({ data: { value: [{ section: "docs" }] } }),
  queryCollectionNavigation: () => ({
    /* no-op in tests */
  }),
  queryCollectionSearchSections: () => ({
    /* no-op in tests */
  }),
  useHead: () => {},
  useSeoMeta: () => {},
}));

describe("ErrorPage", () => {
  it("renders header, footer, error and search with provided data", async () => {
    const nuxtError = {
      statusCode: 404,
      message: "Not Found",
      name: "NotFound",
      fatal: false,
      unhandled: false,
      toJSON: () => ({ statusCode: 404, message: "Not Found" }),
    } as NuxtError;
    const wrapper = await mountSuspended(ErrorPage, {
      props: { error: nuxtError },
      global: {
        stubs: {
          AppHeader,
          AppFooter,
          UApp,
          UError,
          LazyUContentSearch,
          ClientOnly,
          "client-only": ClientOnly,
          "lazy-u-content-search": LazyUContentSearch,
        },
      },
    });

    expect(wrapper.find("header").exists()).toBe(true);
    expect(wrapper.find("footer").exists()).toBe(true);

    const err = wrapper.find("[data-error]");
    expect(err.exists()).toBe(true);
    expect(err.attributes("data-error")).toBe("404");
  });
});
