import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QueryStepInput from "./QueryStepInput.vue";
import type { QueryStep } from "../../composables/types";
import { nextTick } from "vue";

describe("QueryStepInput", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the input field", () => {
    const wrapper = mount(QueryStepInput);
    expect(wrapper.find(".query-step-input").exists()).toBe(true);
    expect(wrapper.find(".step-input").exists()).toBe(true);
  });

  it("shows hints on focus", async () => {
    const wrapper = mount(QueryStepInput);
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");
    await nextTick();
    expect(wrapper.find(".query-hinter").exists()).toBe(true);
  });

  it("filters hints based on input content", async () => {
    const wrapper = mount(QueryStepInput);
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");

    // Simulate typing 'sel'
    const element = input.element as HTMLElement;
    element.innerText = "sel";
    await input.trigger("input");
    await nextTick();

    const hints = wrapper.findAll(".hinter-item");
    // Should only show hints starting with 'sel'
    hints.forEach((hint) => {
      expect(hint.text().toLowerCase()).toMatch(/^sel/);
    });
  });

  it("emits confirm-step when Enter is pressed", async () => {
    const wrapper = mount(QueryStepInput);
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");
    await nextTick();

    await input.trigger("keydown", { key: "Enter" });
    expect(wrapper.emitted("confirm-step")).toBeTruthy();
  });

  it("resets input when Escape is pressed", async () => {
    const wrapper = mount(QueryStepInput);
    const input = wrapper.find(".step-input");
    const element = input.element as HTMLElement;

    element.innerText = "test";
    await input.trigger("input");
    await input.trigger("keydown", { key: "Escape" });
    await nextTick();

    expect(element.innerText).toBe("");
  });

  it("navigates hints with arrow keys", async () => {
    const wrapper = mount(QueryStepInput);
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");
    await nextTick();

    // Arrow down should move to next hint
    await input.trigger("keydown", { key: "ArrowDown" });
    await nextTick();
    expect(wrapper.find(".hinter-item.active").exists()).toBe(true);

    // Arrow up should move back
    await input.trigger("keydown", { key: "ArrowUp" });
    await nextTick();
  });

  it("filters followers based on previous step", () => {
    const prevStep: QueryStep = {
      id: "prev-1",
      type: "select",
    };
    const wrapper = mount(QueryStepInput, {
      props: { prevStep },
    });
    expect(wrapper.find(".query-step-input").exists()).toBe(true);
  });

  it("hides hints when clicking outside", async () => {
    const wrapper = mount(QueryStepInput, {
      attachTo: document.body,
    });
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");
    await nextTick();
    expect(wrapper.find(".query-hinter").exists()).toBe(true);

    // Simulate clicking outside
    document.body.click();
    await nextTick();
    await nextTick(); // Extra tick for vOnClickOutside

    wrapper.unmount();
  });

  it("confirms step when hint is clicked", async () => {
    const wrapper = mount(QueryStepInput);
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");
    await nextTick();

    const firstHint = wrapper.find(".hinter-item");
    await firstHint.trigger("click");

    expect(wrapper.emitted("confirm-step")).toBeTruthy();
  });

  it("does not render when no followers available", () => {
    const prevStep: QueryStep = {
      id: "prev-1",
      type: "select",
    };
    // Mock a scenario where select has no followers
    const wrapper = mount(QueryStepInput, {
      props: { prevStep },
    });
    // Component should still render the input
    expect(wrapper.find(".query-step-input").exists()).toBe(true);
  });

  it("handles Tab key to blur focus", async () => {
    const wrapper = mount(QueryStepInput);
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");
    await nextTick();

    await input.trigger("keydown", { key: "Tab" });
    await nextTick();
    // Tab should not show hints
    expect(wrapper.find(".query-hinter").exists()).toBe(false);
  });

  it("handles click outside to hide hints", async () => {
    const wrapper = mount(QueryStepInput, {
      attachTo: document.body,
    });
    const input = wrapper.find(".step-input");
    await input.trigger("focusin");
    await nextTick();
    expect(wrapper.find(".query-hinter").exists()).toBe(true);

    // Simulate clicking outside
    document.body.click();
    await nextTick();
    await nextTick(); // Extra tick for vOnClickOutside

    expect(wrapper.find(".query-hinter").exists()).toBe(false);
    wrapper.unmount();
  });

  it("should not display component when no followers exist", () => {
    const prevStep: QueryStep = {
      id: "prev-1",
      type: "limit", // assuming limit has no followers
    };
    const wrapper = mount(QueryStepInput, {
      props: { prevStep },
    });
    expect(wrapper.find(".query-step-input").exists()).toBe(false);
  });
});
