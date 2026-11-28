import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ColorRulesEditor from "./ColorRulesEditor.vue";
import { GRAPH_THEME_FALLBACK } from "../../utils/graphStyle";

const ramps = { ...GRAPH_THEME_FALLBACK.ramps, teal: ["#000000", "#ffffff"] };
import type { ColorRule } from "../../utils/graphColorRules";

const palette = ["#111111", "#222222", "#333333"];

const rules: ColorRule[] = [
  {
    kind: "property",
    id: "a",
    key: "role",
    op: "eq",
    value: "admin",
    color: 0,
  },
  { kind: "property", id: "b", key: "name", op: "exists", color: 1 },
  { kind: "palette", id: "c", key: "kind" },
];

const keys = [
  {
    key: "role",
    values: [
      { value: "user", count: 3 },
      { value: "admin", count: 1 },
    ],
  },
  { key: "name", values: [{ value: "alice", count: 1 }] },
];

const mountEditor = (props: Record<string, unknown> = {}) =>
  mount(ColorRulesEditor, {
    attachTo: document.body,
    props: { rules, palette, ramps, keys, ...props },
  });

const lastRules = (wrapper: ReturnType<typeof mountEditor>) =>
  wrapper.emitted("update:rules")?.at(-1)?.[0] as ColorRule[];

const optionTexts = (wrapper: ReturnType<typeof mountEditor>, id: string) =>
  wrapper.findAll(`[data-testid=${id}] option`).map((option) => option.text());

describe("ColorRulesEditor", () => {
  it("lists the rules with readable labels", () => {
    const wrapper = mountEditor();
    expect(
      wrapper.findAll("[data-testid=color-rule]").map((r) => r.text()),
    ).toEqual([
      expect.stringContaining("role = admin"),
      expect.stringContaining("name: any value"),
      expect.stringContaining("kind: each value its own colour"),
    ]);
    // The first rule cannot move up.
    expect(wrapper.findAll("[data-testid=color-rule-up]")).toHaveLength(2);
    wrapper.unmount();
  });

  it("offers the keys and values of the results, with counts", async () => {
    const wrapper = mountEditor({ rules: [] });
    expect(optionTexts(wrapper, "color-rule-key")).toEqual([
      "Property…",
      "role",
      "name",
    ]);
    expect(
      wrapper.get("[data-testid=color-rule-add]").attributes("disabled"),
    ).toBeDefined();
    await wrapper.get("[data-testid=color-rule-key]").setValue("role");
    expect(optionTexts(wrapper, "color-rule-which")).toEqual([
      "a colour per value",
      "any value",
      "= user (3)",
      "= admin (1)",
    ]);
    wrapper.unmount();
  });

  it("adds a colour-per-value rule by default, then value and any-value rules", async () => {
    const wrapper = mountEditor({ rules: [] });
    await wrapper.get("[data-testid=color-rule-key]").setValue("role");
    // No colour to pick for a colour per value.
    expect(wrapper.find("form [data-testid=swatch-trigger]").exists()).toBe(
      false,
    );
    await wrapper.get("form").trigger("submit");
    expect(lastRules(wrapper)[0]).toMatchObject({
      kind: "palette",
      key: "role",
    });

    await wrapper.get("[data-testid=color-rule-which]").setValue("admin");
    // Pick the colour by hand.
    await wrapper.get("form [data-testid=swatch-trigger]").trigger("click");
    await wrapper.findAll("form [data-testid=swatch]")[2]?.trigger("click");
    await wrapper.get("form").trigger("submit");
    expect(lastRules(wrapper)[0]).toMatchObject({
      kind: "property",
      op: "eq",
      value: "admin",
      color: 2,
    });

    await wrapper.get("[data-testid=color-rule-which]").setValue("__any__");
    await wrapper.get("form").trigger("submit");
    const any = lastRules(wrapper)[0];
    // The next colour after the one used last.
    expect(any).toMatchObject({ op: "exists", key: "role", color: 0 });
    expect(any).not.toHaveProperty("value");

    // Choosing another key starts again with a colour per value.
    await wrapper.get("[data-testid=color-rule-key]").setValue("name");
    expect(
      (
        wrapper.get("[data-testid=color-rule-which]")
          .element as HTMLSelectElement
      ).value,
    ).toBe("__each__");
    wrapper.unmount();
  });

  it("explains when there is nothing to colour by", () => {
    const wrapper = mountEditor({ rules: [], keys: [] });
    expect(wrapper.find("form").exists()).toBe(false);
    expect(wrapper.find("[data-testid=color-rule-no-keys]").exists()).toBe(
      true,
    );
    wrapper.unmount();
  });

  it("removes, raises and recolours rules", async () => {
    const wrapper = mountEditor();
    await wrapper
      .findAll("[data-testid=color-rule-remove]")[0]
      ?.trigger("click");
    expect(lastRules(wrapper).map((r) => r.id)).toEqual(["b", "c"]);

    await wrapper.findAll("[data-testid=color-rule-up]")[1]?.trigger("click");
    expect(lastRules(wrapper).map((r) => r.id)).toEqual(["a", "c", "b"]);

    const triggers = wrapper.findAll(
      "[data-testid=color-rule] [data-testid=swatch-trigger]",
    );
    await triggers[1]?.trigger("click");
    await wrapper.findAll("[data-testid=swatch]")[2]?.trigger("click");
    expect(lastRules(wrapper)[1]).toMatchObject({ id: "b", color: 2 });
    expect(lastRules(wrapper)[0]).toEqual(rules[0]);
    wrapper.unmount();
  });

  it("offers a gradient for numeric keys only and lists gradient and distance rules", async () => {
    const wrapper = mountEditor({
      rules: [
        { kind: "gradient", id: "g", key: "age" },
        { kind: "distance", id: "s", from: "start" },
        { kind: "distance", id: "n", from: 4 },
      ],
      keys: [
        { key: "age", numeric: true, values: [{ value: "3", count: 1 }] },
        ...keys,
      ],
      elementLabel: (id: number) => `node ${id} (dave)`,
    });
    expect(
      wrapper.findAll("[data-testid=color-rule]").map((r) => r.text()),
    ).toEqual([
      expect.stringContaining("age: gradient"),
      expect.stringContaining("distance from the search start"),
      expect.stringContaining("distance from node 4 (dave)"),
    ]);
    expect(wrapper.findAll("[data-testid=ramp-trigger]")).toHaveLength(3);

    await wrapper.get("[data-testid=color-rule-key]").setValue("role");
    expect(optionTexts(wrapper, "color-rule-which")).not.toContain(
      "a gradient",
    );
    await wrapper.get("[data-testid=color-rule-key]").setValue("age");
    expect(optionTexts(wrapper, "color-rule-which")).toContain("a gradient");
    await wrapper
      .get("[data-testid=color-rule-which]")
      .setValue("__gradient__");
    // A gradient has no single colour to pick.
    expect(wrapper.find("[data-testid=swatch-trigger]").exists()).toBe(false);
    await wrapper.get("form").trigger("submit");
    expect(lastRules(wrapper).at(-1)).toEqual({
      kind: "gradient",
      id: expect.any(String),
      key: "age",
      // Teal is taken by the rules above.
      scale: "purple",
    });
    wrapper.unmount();
  });

  it("changes the scale of a gradient or distance rule", async () => {
    const wrapper = mountEditor({
      rules: [
        { kind: "palette", id: "p", key: "role" },
        { kind: "distance", id: "d", from: "start" },
      ],
    });
    const trigger = wrapper.get("[data-testid=ramp-trigger]");
    expect(trigger.attributes("aria-label")).toBe("Gradient colours: Teal");
    await trigger.trigger("click");
    const options = wrapper.findAll("[data-testid=ramp-option]");
    expect(options.map((o) => o.text())).toEqual([
      "Teal",
      "Purple",
      "Rose",
      "Several colours",
    ]);
    expect(options[0]?.attributes("aria-pressed")).toBe("true");
    await options[2]?.trigger("click");
    expect(lastRules(wrapper)).toEqual([
      { kind: "palette", id: "p", key: "role" },
      { kind: "distance", id: "d", from: "start", scale: "rose" },
    ]);
    expect(wrapper.find("[data-testid=ramp-option]").exists()).toBe(false);
    wrapper.unmount();
  });

  it("drops the gradient choice when the key is no longer numeric", async () => {
    const age = { key: "age", values: [{ value: "3", count: 1 }] };
    const wrapper = mountEditor({ keys: [{ ...age, numeric: true }] });
    await wrapper.get("[data-testid=color-rule-key]").setValue("age");
    await wrapper
      .get("[data-testid=color-rule-which]")
      .setValue("__gradient__");
    await wrapper.setProps({ keys: [{ ...age, numeric: false }] });
    expect(
      (
        wrapper.get("[data-testid=color-rule-which]")
          .element as HTMLSelectElement
      ).value,
    ).toBe("__each__");
    wrapper.unmount();
  });

  it("labels a distance rule by id without a label function", () => {
    const wrapper = mountEditor({
      rules: [{ kind: "distance", id: "n", from: 7 }],
    });
    expect(wrapper.get("[data-testid=color-rule]").text()).toContain(
      "distance from element 7",
    );
    wrapper.unmount();
  });
});
