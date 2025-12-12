import { mountSuspended, mockNuxtImport } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h, nextTick, ref } from "vue";
import OpenApiCodeBlock from "./OpenApiCodeBlock.vue";

const codeToHtmlMock = vi.hoisted(() =>
  vi.fn(
    async (code: string, opts: { theme?: string }) =>
      `<pre data-theme="${opts?.theme}"><code>${code}</code></pre>`,
  ),
);
vi.mock("shiki", () => ({
  codeToHtml: codeToHtmlMock,
}));

// Provide a reactive color mode and update it during the test
const colorModeRef = ref("light");
mockNuxtImport("useColorMode", () => () => colorModeRef);

vi.mock("../../../../agdb_server/openapi.json", () => ({
  default: { openapi: "3.0.0", info: { title: "agdb", version: "1.0.0" } },
}));

const UButton = defineComponent({
  name: "UButton",
  props: ["size", "variant", "icon", "label"],
  emits: ["click"],
  setup(props, { slots, emit }) {
    return () =>
      h(
        "button",
        { type: "button", onClick: () => emit("click") },
        props.label ?? slots.default?.(),
      );
  },
});

describe("OpenApiCodeBlock copy + theme", () => {
  it("copies code and re-highlights on color mode change", async () => {
    // Mock clipboard
    const writeText = vi.fn();
    vi.stubGlobal("navigator", {
      clipboard: {
        writeText,
      },
    });

    const wrapper = await mountSuspended(OpenApiCodeBlock, {
      global: { stubs: { UButton, "u-button": UButton } },
    });
    await nextTick();

    // Expand to show actions
    const showBtn = wrapper.find("button");
    await showBtn.trigger("click");

    // Click the copy button (first action button)
    const actionButtons = wrapper.findAll(".actions button");
    expect(actionButtons.length).toBeGreaterThanOrEqual(2);
    const copyBtn = actionButtons[0]!;
    await copyBtn.trigger("click");
    expect(writeText).toHaveBeenCalledTimes(1);

    // Verify initial theme, then change to dark and expect re-highlight with dark theme
    expect(codeToHtmlMock.mock.calls[0]?.[1]?.theme).toBe("github-light");

    colorModeRef.value = "dark";
    await nextTick();

    // Another highlight call should have been made with dark theme
    const calls = codeToHtmlMock.mock.calls;
    expect(calls.some(([, opts]) => opts?.theme === "github-dark")).toBe(true);
  });
});
