import { afterEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";

const { addNotification } = vi.hoisted(() => ({ addNotification: vi.fn() }));
vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification,
}));

import ResultsJson from "./ResultsJson.vue";

const results = [
  { result: 1, elements: [{ id: 1, from: 0, to: 0, values: [] }] },
];

describe("ResultsJson", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.clearAllMocks();
  });

  it("renders pretty-printed results", () => {
    const wrapper = mount(ResultsJson, { props: { results } });
    expect(wrapper.find("pre").text()).toBe(JSON.stringify(results, null, 2));
  });

  it("copies the json to the clipboard", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    const wrapper = mount(ResultsJson, { props: { results } });
    await wrapper.find("[data-testid=json-copy]").trigger("click");
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith(JSON.stringify(results, null, 2));
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: "success" }),
    );
  });

  it("reports clipboard failures", async () => {
    vi.stubGlobal("navigator", {
      clipboard: { writeText: vi.fn().mockRejectedValue(new Error("no")) },
    });
    const wrapper = mount(ResultsJson, { props: { results } });
    await wrapper.find("[data-testid=json-copy]").trigger("click");
    await flushPromises();
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: "error" }),
    );
  });
});
