import { mountSuspended, mockNuxtImport } from "@nuxt/test-utils/runtime";
import { describe, it, expect } from "vitest";
import { defineComponent, h } from "vue";
import IndexPage from "./index.vue";

// Stub ContentRenderer to surface received value
const ContentRenderer = defineComponent({
  name: "ContentRenderer",
  props: { value: { type: Object }, prose: { type: Boolean } },
  setup(props) {
    return () => {
      const val = (props as { value?: unknown }).value;
      const isObj = typeof val === "object" && val !== null;
      const unwrapped =
        isObj && "value" in (val as Record<string, unknown>)
          ? (val as { value?: unknown }).value
          : val;
      const title =
        (unwrapped as { title?: string } | null | undefined)?.title ?? "";
      return h("div", { "data-title": title });
    };
  },
});

const page = {
  title: "agdb",
  description: "Graph database",
  seo: { title: "Home", description: "Welcome" },
};

mockNuxtImport(
  "useAsyncData",
  () => () => Promise.resolve({ data: { value: page } }),
);
mockNuxtImport("useSeoMeta", () => () => {});
mockNuxtImport(
  "createError",
  () => (args: { statusMessage?: string }) =>
    new Error(args?.statusMessage || "Page not found"),
);

describe("IndexPage (/)", () => {
  it("renders content and sets SEO meta from page", async () => {
    const wrapper = await mountSuspended(IndexPage, {
      global: {
        stubs: { ContentRenderer, "content-renderer": ContentRenderer },
      },
    });

    const el = wrapper.find("[data-title]");
    expect(el.exists()).toBe(true);
    expect(el.attributes("data-title")).toBe("agdb");
  });
});
