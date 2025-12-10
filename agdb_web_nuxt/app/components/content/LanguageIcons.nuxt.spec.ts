import { mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, it, expect } from "vitest";
import { defineComponent, h } from "vue";
import LanguageIcons from "./LanguageIcons.vue";

const NuxtLink = defineComponent({
  name: "NuxtLink",
  props: { to: { type: String, required: true } },
  setup(props, { slots }) {
    return () => h("a", { href: props.to }, slots.default?.());
  },
});

describe("LanguageIcons", () => {
  it("renders links to API docs languages", async () => {
    const wrapper = await mountSuspended(LanguageIcons, {
      global: { stubs: { NuxtLink } },
    });

    const rust = wrapper.find('a[href="/api-docs/rust"]');
    const ts = wrapper.find('a[href="/api-docs/typescript"]');
    const php = wrapper.find('a[href="/api-docs/php"]');

    expect(rust.exists()).toBe(true);
    expect(ts.exists()).toBe(true);
    expect(php.exists()).toBe(true);

    const imgs = wrapper.findAll("img");
    expect(imgs.length).toBeGreaterThan(3);
  });
});
