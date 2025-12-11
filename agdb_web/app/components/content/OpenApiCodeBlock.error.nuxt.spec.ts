import { mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, it, expect, vi } from "vitest";
import { defineComponent, h, nextTick } from "vue";
import OpenApiCodeBlock from "./OpenApiCodeBlock.vue";

// Mock shiki but we won't reach it due to stringify error
vi.mock("shiki", () => ({
  codeToHtml: vi.fn(async () => "<pre></pre>"),
}));

// Cause JSON.stringify to throw by including a BigInt
vi.mock("../../../../agdb_server/openapi.json", () => ({
  default: { broken: BigInt(1) },
}));

const UButton = defineComponent({
  name: "UButton",
  emits: ["click"],
  setup(_, { slots, emit }) {
    return () =>
      h(
        "button",
        { type: "button", onClick: () => emit("click") },
        slots.default?.(),
      );
  },
});

describe("OpenApiCodeBlock error", () => {
  it("logs an error when openapi fails to stringify", async () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    await mountSuspended(OpenApiCodeBlock, {
      global: { stubs: { UButton, "u-button": UButton } },
    });
    await nextTick();
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });
});
