import { mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h, ref, computed } from "vue";
import DocsLayout from "./docs.vue";

const { useRoute } = vi.hoisted(() => {
  return {
    useRoute: vi.fn(() => ({ path: "/docs/examples" })),
  };
});

vi.mock("vue-router", async (orig) => {
  const mod = await orig();
  return {
    ...(mod as object),
    useRoute,
  };
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
    return () => h("div", [slots.left?.(), slots.default?.()]);
  },
});
const UPageAside = defineComponent({
  name: "UPageAside",
  setup(_, { slots }) {
    return () => h("aside", slots.default?.());
  },
});

// This stub inspects the navigation prop to expose the examples children length
const UContentNavigation = defineComponent({
  name: "UContentNavigation",
  props: { navigation: { type: Array, default: () => [] }, highlight: Boolean },
  setup(props) {
    const count = computed(() => {
      const list = props.navigation as Array<{
        title?: string;
        children?: unknown[];
      }>;
      const examples = list.find((i) => i?.title === "Examples");
      return Array.isArray(examples?.children)
        ? (examples.children as unknown[]).length
        : 0;
    });
    const hasExternal = computed(() => {
      const list = props.navigation as Array<{
        title?: string;
        children?: Array<{ path?: string }>;
      }>;
      const examples = list.find((i) => i?.title === "Examples");
      return Boolean(
        examples?.children?.some(
          (c) => typeof c.path === "string" && c.path.startsWith("http"),
        ),
      );
    });
    const nonExamplesUnchanged = computed(() => {
      const list = props.navigation as Array<{
        title?: string;
        children?: Array<{ path?: string }>;
      }>;
      return list
        .filter((i) => i?.title !== "Examples")
        .every(
          (item) =>
            !item.children?.some(
              (c) => typeof c.path === "string" && c.path.startsWith("http"),
            ),
        );
    });
    return () =>
      h("div", {
        "data-examples-count": String(count.value),
        "data-examples-external": String(hasExternal.value),
        "data-non-examples-unchanged": String(nonExamplesUnchanged.value),
      });
  },
});

describe("DocsLayout examples mapping", () => {
  it("injects external links for /docs/examples under /docs top section", async () => {
    console.log("Starting test for /docs/examples");
    useRoute.mockReturnValue({ path: "/docs/examples" });

    const navigation = [
      {
        title: "Docs",
        path: "/docs",
        children: [
          { title: "Guides", path: "/docs/guides" },
          {
            title: "Examples",
            path: "/docs/examples",
          },
          { title: "References", path: "/docs/references" },
        ],
      },
    ];

    const wrapper = await mountSuspended(DocsLayout, {
      global: {
        provide: { navigation: ref(navigation) },
        stubs: { UContainer, UPage, UPageAside, UContentNavigation },
      },
    });

    const el = wrapper.find("[data-examples-count]");
    expect(el.exists()).toBe(true);
    expect(el.attributes("data-examples-external")).toBe("true");
    expect(el.attributes("data-non-examples-unchanged")).toBe("true");
    expect(Number(el.attributes("data-examples-count"))).toBeGreaterThan(0);
  });

  it("handles other top sections without mapping", async () => {
    // Switch route to API docs
    useRoute.mockReturnValue({ path: "/api-docs" });
    const navigation = [
      {
        title: "API",
        path: "/api-docs",
        children: [{ title: "OpenAPI", path: "/api-docs/openapi" }],
      },
    ];

    const wrapper = await mountSuspended(DocsLayout, {
      global: {
        provide: { navigation: ref(navigation) },
        stubs: { UContainer, UPage, UPageAside, UContentNavigation },
      },
    });

    const el = wrapper.find("[data-examples-count]");
    expect(el.exists()).toBe(true);
    expect(Number(el.attributes("data-examples-count"))).toBe(0);
  });

  it("handles missing top section (no children)", async () => {
    // Route points to /docs but navigation has no matching section
    useRoute.mockReturnValue({ path: "/docs/anything" });
    const navigation: Array<{
      title?: string;
      path?: string;
      children?: unknown[];
    }> = [];

    const wrapper = await mountSuspended(DocsLayout, {
      global: {
        provide: { navigation: ref(navigation) },
        stubs: { UContainer, UPage, UPageAside, UContentNavigation },
      },
    });

    const el = wrapper.find("[data-examples-count]");
    expect(el.exists()).toBe(true);
    expect(Number(el.attributes("data-examples-count"))).toBe(0);
  });

  it("handles missing navigation injection (defaults to empty)", async () => {
    useRoute.mockReturnValue({ path: "/docs/examples" });

    const wrapper = await mountSuspended(DocsLayout, {
      global: {
        // No navigation provided
        stubs: { UContainer, UPage, UPageAside, UContentNavigation },
      },
    });

    const el = wrapper.find("[data-examples-count]");
    expect(el.exists()).toBe(true);
    expect(Number(el.attributes("data-examples-count"))).toBe(0);
  });
});
