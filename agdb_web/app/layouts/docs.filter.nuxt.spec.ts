import { mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h, ref } from "vue";
import DocsLayout from "./docs.vue";

// Stub UContentNavigation to capture navigation prop length
const UContentNavigation = defineComponent({
  name: "UContentNavigation",
  props: { navigation: { type: Array, default: () => [] }, highlight: Boolean },
  setup(props) {
    return () =>
      h("div", { "data-count": (props.navigation as unknown[]).length });
  },
});

const UContainer = defineComponent({
  name: "UContainer",
  setup(_, { slots }) {
    return () => h("div", slots.default?.());
  },
});
const UPage = defineComponent({
  name: "UPage",
  setup(_, { slots }) {
    // Render both default and named `left` slot so aside content mounts
    return () => h("div", [slots.left?.(), slots.default?.()]);
  },
});
const UPageAside = defineComponent({
  name: "UPageAside",
  setup(_, { slots }) {
    return () => h("aside", slots.default?.());
  },
});

describe("DocsLayout scoped nav", () => {
  it("shows children of the current top-level section (/docs)", async () => {
    // Mock the route
    vi.mock("vue-router", async (orig) => {
      const mod = await orig();
      return {
        ...(mod as object),
        useRoute: () => ({ path: "/docs/guides/how-to-run-server" }),
      };
    });

    const navigation = [
      {
        title: "Docs",
        path: "/docs",
        children: [
          { title: "Guides", path: "/docs/guides" },
          { title: "Examples", path: "/docs/examples" },
          { title: "References", path: "/docs/references" },
        ],
      },
      {
        title: "API",
        path: "/api-docs",
        children: [{ title: "OpenAPI", path: "/api-docs/openapi" }],
      },
    ];

    const wrapper = await mountSuspended(DocsLayout, {
      global: {
        // The layout injects a Ref<ContentNavigationItem[]> under key "navigation"
        provide: { navigation: ref(navigation) },
        stubs: { UContainer, UPage, UPageAside, UContentNavigation },
      },
    });

    const el = wrapper.find("[data-count]");
    expect(el.exists()).toBe(true);
    expect(Number(el.attributes("data-count"))).toBe(3);
  });
});
