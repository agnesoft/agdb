import { mockNuxtImport, mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h, nextTick } from "vue";
import OpenApiCodeBlock from "./OpenApiCodeBlock.vue";

vi.mock("shiki", () => ({
  codeToHtml: vi.fn(async (code: string) => `<pre><code>${code}</code></pre>`),
}));

const colorModeRef = ref("light");
mockNuxtImport("useColorMode", () => () => colorModeRef);

// Mock dynamic import of openapi.json (match specifier used in component)
vi.mock("../../../../agdb_server/openapi.json", () => ({
  default: { openapi: "3.0.0", info: { title: "agdb", version: "1.0.0" } },
}));

// Simple stub for UButton to behave like a native button
const UButton = defineComponent({
  name: "UButton",
  props: ["size", "variant", "icon", "label"],
  emits: ["click"],
  setup(props, { slots, emit }) {
    return () =>
      h(
        "button",
        { type: "button", onClick: () => emit("click") },
        // Render label prop if provided, else default slot
        props.label ?? slots.default?.(),
      );
  },
});

describe("OpenApiCodeBlock", () => {
  it("loads openapi.json and toggles expand/collapse", async () => {
    const testJson = {
      openapi: "3.0.0",
      info: { title: "agdb", version: "1.0.0" },
    };
    const wrapper = await mountSuspended(OpenApiCodeBlock, {
      props: { testCode: JSON.stringify(testJson, null, 2) },
      global: { stubs: { UButton, "u-button": UButton } },
    });

    await nextTick();

    expect(wrapper.find("button").exists()).toBe(true);
    expect(wrapper.text()).toMatch(/Show openapi.json/i);

    await wrapper.find("button").trigger("click");
    expect(wrapper.text()).toMatch(/openapi.json/i);

    const buttons = wrapper.findAll('[data-testid="hide-button"]');
    expect(buttons.length).toBe(1);
    expect(buttons[0]?.text()).toMatch(/Hide/i);

    // Collapse again
    await buttons[0]?.trigger("click");
    expect(wrapper.text()).toMatch(/Show openapi.json/i);
  });
});
