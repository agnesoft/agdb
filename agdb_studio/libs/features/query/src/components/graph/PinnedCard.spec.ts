import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import { mount } from "@vue/test-utils";
import PinnedCard from "./PinnedCard.vue";
import type { GraphLink, GraphNode } from "../../composables/types";

const node: GraphNode = {
  id: 4,
  alias: "dave",
  values: [{ key: "name", value: "Dave" }],
  rawValues: [],
  queryOnlyValues: [{ key: { String: "score" }, value: { I64: 7 } }],
  source: "query",
};
const link: GraphLink = {
  id: -5,
  values: [],
  rawValues: [],
  source: 1,
  target: 2,
  elementSource: "query",
};

describe("PinnedCard", () => {
  it("shows the node, its alias, values and query-only values", () => {
    const wrapper = mount(PinnedCard, {
      props: { element: node, x: 10, y: 20 },
    });
    expect(wrapper.attributes("style")).toContain("translate(10px, 20px)");
    expect(wrapper.get(".card-title").text()).toBe("Node 4 dave");
    const rows = wrapper.findAll(".card-row").map((r) => r.text());
    expect(rows).toEqual(["name Dave", "score 7 query"]);
  });

  it("shows the endpoints of an edge and its loading state", async () => {
    const wrapper = mount(PinnedCard, {
      props: { element: { ...link, valuesState: "loading" }, x: 0, y: 0 },
    });
    expect(wrapper.get(".card-title").text()).toBe("Edge -5");
    expect(wrapper.get(".card-endpoints").text()).toBe("1 → 2");
    expect(wrapper.get(".card-empty").text()).toBe("Loading…");
    await wrapper.setProps({ element: link });
    expect(wrapper.get(".card-empty").text()).toBe("No values");
  });

  it("emits unpin, open and drag-start", async () => {
    const wrapper = mount(PinnedCard, { props: { element: node, x: 0, y: 0 } });
    const unpin = wrapper.get("[data-testid=pinned-card-unpin]");
    // Pressing the button does not start dragging the card.
    unpin.element.dispatchEvent(
      new MouseEvent("pointerdown", { bubbles: true, button: 0 }),
    );
    await unpin.trigger("click");
    const header = wrapper.get(".card-header");
    await header.trigger("dblclick");
    // jsdom has no PointerEvent.
    header.element.dispatchEvent(
      new MouseEvent("pointerdown", { bubbles: true, button: 0 }),
    );
    expect(wrapper.emitted("unpin")).toHaveLength(1);
    expect(wrapper.emitted("open")).toHaveLength(1);
    expect(wrapper.emitted("drag-start")).toHaveLength(1);
  });
});
