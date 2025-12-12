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

// Hoisted mocks for the error path
mockNuxtImport(
  "useAsyncData",
  () => () =>
    Promise.resolve({
      data: { value: null },
    }),
);
mockNuxtImport("useSeoMeta", () => () => {});
mockNuxtImport(
  "createError",
  () => (args: { statusMessage?: string }) =>
    new Error(args?.statusMessage || "Page not found"),
);

describe("IndexPage (error path)", () => {
  it("throws 404 when no page is found", async () => {
    let captured: unknown;
    await mountSuspended(IndexPage, {
      global: {
        config: {
          errorHandler: (err) => {
            captured = err;
            return false;
          },
        },
        stubs: { ContentRenderer, "content-renderer": ContentRenderer },
      },
    });
    expect((captured as Error).message).toBe("Page not found");
  });
});
