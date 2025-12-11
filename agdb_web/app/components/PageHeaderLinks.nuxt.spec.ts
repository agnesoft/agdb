import { describe, it, expect, vi } from "vitest";

import PageHeaderLinks from "./PageHeaderLinks.vue";
import { mockNuxtImport, mountSuspended } from "@nuxt/test-utils/runtime";
import { defineComponent, h } from "vue";

const UIcon = defineComponent({
  name: "UIcon",
  props: {
    name: { type: String, required: true },
  },
  setup(props) {
    return () => h("i", `icon:${props.name}`);
  },
});

mockNuxtImport("useToast", () => () => ({
  show: vi.fn(),
}));

vi.mock("@vueuse/core", async (importOriginal) => {
  const original = await importOriginal();
  return {
    ...(original as object),
    useClipboard: () => ({
      copy: vi.fn().mockResolvedValue(true),
    }),
  };
});

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

vi.stubGlobal("useSiteConfig", () => ({
  url: "https://agdb.io",
  header: { links: [] },
}));

describe("PageHeaderLinks", () => {
  it("renders links with icons", async () => {
    const wrapper = await mountSuspended(PageHeaderLinks, {
      props: {
        links: [
          { href: "https://example.com", icon: "home", label: "Home" },
          { href: "https://github.com", icon: "github", label: "GitHub" },
        ],
      },
      global: {
        stubs: {
          UIcon,
          "u-icon": UIcon,
          UFieldGroup: { template: "<div><slot /></div>" },
          UButton: {
            emits: ["click"],
            props: [
              "label",
              "icon",
              "color",
              "variant",
              "ui",
              "size",
              "ariaLabel",
            ],
            template:
              "<button @click=\"$emit('click')\"><slot />{{ label }}</button>",
          },
          UDropdownMenu: {
            props: ["items", "content", "ui"],
            template: `<div class="dropdown" :data-items-count="items?.length">
                <slot />
                <template v-for="it in items">
                  <a v-if="it.to" :href="it.to" :target="it.target">{{ it.label }}</a>
                  <a v-else-if="it.href" :href="it.href">{{ it.label }}</a>
                  <button v-else @click="it.onSelect?.()">{{ it.label }}</button>
                </template>
              </div>`,
          },
        },
      },
    });

    const linkElements = wrapper.findAll("a");
    // At least 3 links from dropdown items (Markdown, ChatGPT, Claude)
    expect(linkElements.length).toBeGreaterThanOrEqual(3);
    // Verify presence of ChatGPT and Claude links
    expect(
      linkElements.some((a) =>
        a.attributes("href")?.startsWith("https://chatgpt.com"),
      ),
    ).toBe(true);
    expect(
      linkElements.some((a) =>
        a.attributes("href")?.startsWith("https://claude.ai"),
      ),
    ).toBe(true);
  });

  it("renders no links when none are provided", async () => {
    const wrapper = await mountSuspended(PageHeaderLinks, {
      props: { links: [] },
      global: {
        stubs: {
          UIcon,
          "u-icon": UIcon,
          UFieldGroup: { template: "<div><slot /></div>" },
          UButton: { template: "<button><slot /></button>" },
          UDropdownMenu: { template: '<div class="dropdown"><slot /></div>' },
        },
      },
    });

    const linkElements = wrapper.findAll("a");
    expect(linkElements).toHaveLength(0);
  });
});
