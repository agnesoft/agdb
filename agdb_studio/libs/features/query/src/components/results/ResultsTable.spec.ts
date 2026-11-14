import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import { mount } from "@vue/test-utils";
import ResultsTable from "./ResultsTable.vue";

const s = (value: string) => ({ String: value });

describe("ResultsTable", () => {
  it("renders one column per distinct key and edge endpoints", () => {
    const wrapper = mount(ResultsTable, {
      props: {
        results: [
          {
            result: 2,
            elements: [
              {
                id: 1,
                from: 0,
                to: 0,
                values: [{ key: s("name"), value: s("A") }],
              },
              {
                id: -2,
                from: 1,
                to: 3,
                values: [{ key: s("w"), value: { I64: 5 } }],
              },
            ],
          },
        ],
      },
    });
    const headers = wrapper.findAll("th").map((th) => th.text());
    expect(headers).toEqual(["", "id", "from", "to", "name", "w"]);
    const rows = wrapper.findAll("[data-testid=results-table-row]");
    expect(rows[0]?.findAll("td").map((td) => td.text())).toEqual([
      "⋮",
      "1",
      "",
      "",
      "A",
      "",
    ]);
    expect(rows[1]?.findAll("td").map((td) => td.text())).toEqual([
      "⋮",
      "-2",
      "1",
      "3",
      "",
      "5",
    ]);
  });

  it("labels id 0 rows and hides their menu", async () => {
    const wrapper = mount(ResultsTable, {
      props: {
        results: [
          {
            result: 1,
            elements: [
              {
                id: 0,
                from: 0,
                to: 0,
                values: [{ key: s("idx"), value: { U64: 3 } }],
              },
            ],
          },
        ],
      },
    });
    expect(wrapper.find("th").exists()).toBe(true);
    expect(wrapper.findAll("th").map((th) => th.text())).toEqual([
      "",
      "id",
      "idx",
    ]);
    expect(wrapper.text()).toContain("(database)");
    expect(wrapper.find("[data-testid=results-row-menu]").exists()).toBe(false);
    await wrapper
      .find("[data-testid=results-table-row]")
      .trigger("contextmenu");
    expect(wrapper.emitted("menu")).toBeUndefined();
  });

  it("emits menu events from the button and context menu", async () => {
    const element = { id: 1, from: 0, to: 0, values: [] };
    const wrapper = mount(ResultsTable, {
      props: { results: [{ result: 1, elements: [element] }] },
    });
    await wrapper
      .find("[data-testid=results-row-menu]")
      .trigger("click", { clientX: 3, clientY: 4 });
    await wrapper
      .find("[data-testid=results-table-row]")
      .trigger("contextmenu", { clientX: 5, clientY: 6 });
    expect(wrapper.emitted("menu")).toEqual([
      [element, 3, 4],
      [element, 5, 6],
    ]);
  });

  it("shows an empty state", () => {
    const wrapper = mount(ResultsTable, { props: { results: [] } });
    expect(wrapper.text()).toContain("No elements.");
  });
});
