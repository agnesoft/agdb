import { mountSuspended, mockNuxtImport } from "@nuxt/test-utils/runtime";
import { describe, it, expect } from "vitest";
import { defineComponent, h, nextTick } from "vue";
import AppFooter from "./AppFooter.vue";

const appConfig = {
  footer: {
    credits: "Copyright © 2026 agdb",
    colorMode: false,
    links: undefined as
      | Array<{ href: string; label: string }>
      | undefined,
  },
};

mockNuxtImport("useAppConfig", () => () => appConfig);

// Minimal stubs for UI components
const UFooter = defineComponent({
  name: "UFooter",
  setup(_, { slots }) {
    return () =>
      h("footer", [slots.left?.(), slots.default?.(), slots.right?.()]);
  },
});
const UColorModeButton = defineComponent({
  name: "UColorModeButton",
  setup: () => () => h("button", { "data-color-mode": "btn" }),
});
const UButton = defineComponent({
  name: "UButton",
  props: ["color", "variant", "href", "to", "label"],
  setup(props) {
    return () =>
      h("a", { "data-link": props.href ?? props.to ?? "" }, props.label ?? "");
  },
});

describe("AppFooter", () => {
  it("renders credits and optional color mode button", async () => {
    appConfig.footer = {
      credits: "Copyright © 2026 agdb",
      colorMode: true,
      links: [],
    };

    const wrapper = await mountSuspended(AppFooter, {
      global: {
        stubs: {
          UFooter,
          UColorModeButton,
          UButton,
          "u-footer": UFooter,
          "u-color-mode-button": UColorModeButton,
          "u-button": UButton,
        },
      },
    });

    await nextTick();
    expect(wrapper.find("footer").exists()).toBe(true);
    expect(wrapper.text()).toContain("Copyright ©");
  });

  it("renders provided links when present", async () => {
    appConfig.footer = {
      credits: "Copyright © 2026 agdb",
      colorMode: false,
      links: [
        { href: "https://github.com/agnesoft", label: "GitHub" },
        { href: "https://agnesoft.com", label: "Website" },
      ],
    };

    const wrapper = await mountSuspended(AppFooter, {
      global: {
        stubs: {
          UFooter,
          UColorModeButton,
          UButton,
          "u-footer": UFooter,
          "u-color-mode-button": UColorModeButton,
          "u-button": UButton,
        },
      },
    });

    await nextTick();
    const links = wrapper.findAll("[data-link]");
    expect(links.length).toBeGreaterThanOrEqual(1);
    expect(links[0]?.attributes("data-link")).toBe(
      "https://github.com/agnesoft",
    );
    expect(links[1]?.attributes("data-link")).toBe("https://agnesoft.com");
  });

  it("omits optional controls when disabled", async () => {
    appConfig.footer = {
      credits: "Copyright © 2026 agdb",
      colorMode: false,
      links: undefined,
    };

    const wrapper = await mountSuspended(AppFooter, {
      global: {
        stubs: {
          UFooter,
          UColorModeButton,
          UButton,
          "u-footer": UFooter,
          "u-color-mode-button": UColorModeButton,
          "u-button": UButton,
        },
      },
    });

    await nextTick();
    expect(wrapper.find("[data-color-mode='btn']").exists()).toBe(false);
    expect(wrapper.findAll("[data-link]")).toHaveLength(0);
  });
});
