import { mountSuspended, mockNuxtImport } from "@nuxt/test-utils/runtime";
import AppHeader from "./AppHeader.vue";
import { describe, it, expect } from "vitest";
import { defineComponent, h, ref } from "vue";

const appConfig = {
  header: {} as Record<string, unknown>,
};

mockNuxtImport("useAppConfig", () => () => appConfig);

const UHeader = defineComponent({
  name: "UHeader",
  props: ["to"],
  setup(props, { slots }) {
    return () =>
      h("header", { "data-to": props.to as string | undefined }, [
        slots.default?.(),
        slots.left?.(),
        slots.title?.(),
        slots.right?.(),
        slots.body?.(),
      ]);
  },
});

const UContentSearchButton = defineComponent({
  name: "UContentSearchButton",
  props: ["collapsed"],
  setup(props) {
    return () =>
      h("button", {
        "data-search": String(props.collapsed ?? "expanded"),
      });
  },
});

const UColorModeImage = defineComponent({
  name: "UColorModeImage",
  props: ["light", "dark", "alt"],
  setup(props) {
    return () =>
      h("img", {
        "data-light": props.light as string | undefined,
        "data-dark": props.dark as string | undefined,
        alt: props.alt as string | undefined,
      });
  },
});

const UButton = defineComponent({
  name: "UButton",
  props: ["label", "to", "href"],
  setup(props) {
    return () =>
      h(
        "a",
        { "data-link": (props.to as string | undefined) ?? (props.href as string | undefined) ?? "" },
        props.label as string | undefined,
      );
  },
});

const UContentNavigation = defineComponent({
  name: "UContentNavigation",
  props: ["navigation"],
  setup(props) {
    return () =>
      h(
        "nav",
        { "data-navigation-items": String(((props.navigation as unknown[]) ?? []).length) },
      );
  },
});

const UColorModeButton = defineComponent({
  name: "UColorModeButton",
  setup() {
    return () => h("button", { "data-color-mode": "true" });
  },
});

const NuxtLink = defineComponent({
  name: "NuxtLink",
  props: ["to"],
  setup(props, { slots }) {
    return () =>
      h("a", { href: props.to as string | undefined }, slots.default?.());
  },
});

const AppLogo = defineComponent({
  name: "AppLogo",
  setup() {
    return () => h("div", { "data-logo": "app-logo" });
  },
});

const globalStubs = {
  UHeader,
  UContentSearchButton,
  UColorModeImage,
  UButton,
  UContentNavigation,
  UColorModeButton,
  NuxtLink,
  AppLogo,
};

describe("AppHeader", () => {
  it("renders navigation links", async () => {
    appConfig.header = {
      to: "/docs",
      title: "agdb docs",
      search: true,
      colorMode: true,
      internalLinks: [{ label: "Guide", to: "/guide" }],
      links: [{ label: "GitHub", href: "https://github.com/agnesoft/agdb" }],
    };

    const wrapper = await mountSuspended(AppHeader, {
      global: {
        provide: { navigation: ref([{ path: "/docs" }]) },
        stubs: globalStubs,
      },
    });

    expect(wrapper.find("header").attributes("data-to")).toBe("/docs");
    expect(wrapper.text()).toContain("agdb docs");
    expect(wrapper.findAll("button[data-search]")).toHaveLength(2);
    expect(wrapper.find("button[data-color-mode='true']").exists()).toBe(true);
    expect(wrapper.find("a[data-link='/guide']").exists()).toBe(true);
    expect(
      wrapper
        .find("a[data-link='https://github.com/agnesoft/agdb']")
        .exists(),
    ).toBe(true);
    expect(wrapper.find("nav").attributes("data-navigation-items")).toBe("1");
  });

  it("renders the configured logo when available", async () => {
    appConfig.header = {
      logo: {
        light: "/logo-light.svg",
        dark: "/logo-dark.svg",
        alt: "agdb",
      },
    };

    const wrapper = await mountSuspended(AppHeader, {
      global: {
        provide: { navigation: ref([]) },
        stubs: globalStubs,
      },
    });

    expect(wrapper.find("img[alt='agdb']").exists()).toBe(true);
  });

  it("falls back to AppLogo when neither title nor logo is configured", async () => {
    appConfig.header = {
      to: "/",
    };

    const wrapper = await mountSuspended(AppHeader, {
      global: {
        provide: { navigation: ref([]) },
        stubs: globalStubs,
      },
    });

    expect(wrapper.find("[data-logo='app-logo']").exists()).toBe(true);
    expect(wrapper.find("a[href='/']").exists()).toBe(true);
  });
});
