import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ChartSettings from "./ChartSettings.vue";
import type { ChartConfig, ResultKey } from "../../../utils/chartConfig";

const keys: ResultKey[] = [
  { key: "type", count: 3, numeric: false, textual: true, distinct: 2 },
  { key: "price", count: 3, numeric: true, textual: false, distinct: 3 },
  { key: "weight", count: 2, numeric: true, textual: false, distinct: 2 },
];

const countByType: ChartConfig = {
  kind: "bar",
  groupBy: { by: "property", key: "type" },
  measure: { aggregate: "count" },
};

const mountSettings = (config: ChartConfig = countByType, k = keys) =>
  mount(ChartSettings, { props: { config, keys: k } });

const lastUpdate = (wrapper: ReturnType<typeof mountSettings>) =>
  wrapper.emitted<[ChartConfig]>("update:config")?.at(-1)?.[0];

const options = (wrapper: ReturnType<typeof mountSettings>, testId: string) =>
  wrapper
    .find(`[data-testid="${testId}"]`)
    .findAll("option")
    .map((o) => o.text());

describe("ChartSettings", () => {
  it("switches the chart type", async () => {
    const wrapper = mountSettings();
    expect(
      wrapper.find('[data-testid="chart-kind-bar"]').attributes("aria-pressed"),
    ).toBe("true");
    await wrapper.find('[data-testid="chart-kind-donut"]').trigger("click");
    expect(lastUpdate(wrapper)).toEqual({ ...countByType, kind: "donut" });
  });

  it("offers grouping by query or any property", async () => {
    const wrapper = mountSettings();
    expect(options(wrapper, "chart-group-by")).toEqual([
      "Query of the batch",
      "type",
      "price",
      "weight",
    ]);
    const select = wrapper.find('[data-testid="chart-group-by"]');
    await select.setValue("price");
    expect(lastUpdate(wrapper)?.groupBy).toEqual({
      by: "property",
      key: "price",
    });
    await select.setValue(select.findAll("option")[0]!.element.value);
    expect(lastUpdate(wrapper)?.groupBy).toEqual({ by: "query" });
  });

  it("measures numeric properties only, keeping the key between aggregates", async () => {
    const wrapper = mountSettings();
    expect(wrapper.find('[data-testid="chart-measure-key"]').exists()).toBe(
      false,
    );
    await wrapper.find('[data-testid="chart-aggregate"]').setValue("sum");
    const sum = lastUpdate(wrapper)!;
    expect(sum.measure).toEqual({ aggregate: "sum", key: "price" });

    await wrapper.setProps({ config: sum });
    expect(options(wrapper, "chart-measure-key")).toEqual(["price", "weight"]);
    await wrapper.find('[data-testid="chart-measure-key"]').setValue("weight");
    expect(lastUpdate(wrapper)?.measure).toEqual({
      aggregate: "sum",
      key: "weight",
    });

    await wrapper.setProps({
      config: { ...sum, measure: { aggregate: "sum", key: "weight" } },
    });
    await wrapper.find('[data-testid="chart-aggregate"]').setValue("max");
    expect(lastUpdate(wrapper)?.measure).toEqual({
      aggregate: "max",
      key: "weight",
    });
    await wrapper.find('[data-testid="chart-aggregate"]').setValue("count");
    expect(lastUpdate(wrapper)?.measure).toEqual({ aggregate: "count" });
  });

  it("disables sums without numeric properties, and says why", () => {
    const wrapper = mountSettings(countByType, [keys[0]!]);
    const disabled = wrapper
      .find('[data-testid="chart-aggregate"]')
      .findAll("option")
      .filter((o) => o.attributes("disabled") !== undefined)
      .map((o) => o.text());
    expect(disabled).toEqual(["Sum", "Average", "Min", "Max"]);
    expect(wrapper.find(".note").exists()).toBe(true);
    // No note before there are any keys (e.g. before the first run).
    expect(mountSettings(countByType, []).find(".note").exists()).toBe(false);
  });

  it("keeps a donut to measures that add up", async () => {
    const avg: ChartConfig = {
      ...countByType,
      measure: { aggregate: "avg", key: "price" },
    };
    const bar = mountSettings(avg);
    await bar.find('[data-testid="chart-kind-donut"]').trigger("click");
    // Stored as asked: the donut shows the sum, a bar the average again.
    expect(lastUpdate(bar)).toEqual({ ...avg, kind: "donut" });
    const shown = mount(ChartSettings, {
      props: {
        config: {
          ...avg,
          kind: "donut",
          measure: { aggregate: "sum", key: "price" },
        },
        saved: { ...avg, kind: "donut" },
        keys,
      },
    });
    expect(
      shown.find<HTMLSelectElement>('[data-testid="chart-aggregate"]').element
        .value,
    ).toBe("sum");
    await shown.find('[data-testid="chart-kind-bar"]').trigger("click");
    expect(lastUpdate(shown)).toEqual(avg);
    // Another key keeps the stored average.
    await shown.find('[data-testid="chart-measure-key"]').setValue("weight");
    expect(lastUpdate(shown)).toEqual({
      ...avg,
      kind: "donut",
      measure: { aggregate: "avg", key: "weight" },
    });
    const donut = mountSettings({ ...countByType, kind: "donut" });
    const disabled = donut
      .find('[data-testid="chart-aggregate"]')
      .findAll("option")
      .filter((o) => o.attributes("disabled") !== undefined);
    expect(disabled.map((o) => o.text())).toEqual(["Average", "Min", "Max"]);
    expect(disabled[0]!.attributes("title")).toMatch(/bar chart/);
  });

  it("splits a line by property into a line per query of a batch", async () => {
    const line: ChartConfig = { ...countByType, kind: "line" };
    // One query: nothing to split.
    expect(
      mountSettings(line).find('[data-testid="chart-split"]').exists(),
    ).toBe(false);
    const wrapper = mount(ChartSettings, {
      props: { config: line, keys, queries: 3 },
    });
    const split = wrapper.find('[data-testid="chart-split"]');
    await split.setValue("query");
    const splitLine = lastUpdate(wrapper)!;
    expect(splitLine).toEqual({ ...line, split: { by: "query" } });
    await wrapper.setProps({ config: splitLine, queries: 1 });
    // Kept on show while set, so it can be undone.
    await wrapper.find('[data-testid="chart-split"]').setValue("");
    expect(lastUpdate(wrapper)).toEqual(line);
    expect(lastUpdate(wrapper)).not.toHaveProperty("split");
    // Bars do not split (yet the split is kept for going back to lines).
    await wrapper.setProps({
      config: { ...splitLine, kind: "bar" },
      queries: 3,
    });
    expect(wrapper.find('[data-testid="chart-split"]').exists()).toBe(false);
  });

  it("draws queries of the batch as bars, not a line", async () => {
    const wrapper = mountSettings({ ...countByType, kind: "line" });
    const select = wrapper.find('[data-testid="chart-group-by"]');
    await select.setValue(select.findAll("option")[0]!.element.value);
    // Stored as a line, shown as bars until grouped by a property again.
    expect(lastUpdate(wrapper)).toMatchObject({
      kind: "line",
      groupBy: { by: "query" },
    });
    const byQuery = mountSettings({ ...countByType, groupBy: { by: "query" } });
    const lineKind = byQuery.find('[data-testid="chart-kind-line"]');
    expect(lineKind.attributes("disabled")).toBeDefined();
    expect(lineKind.attributes("title")).toMatch(/group by a property/);
  });

  it("keeps keys of the config the results do not have", () => {
    const wrapper = mountSettings(
      {
        kind: "line",
        groupBy: { by: "property", key: "day" },
        measure: { aggregate: "avg", key: "load" },
      },
      [],
    );
    expect(options(wrapper, "chart-group-by")).toEqual([
      "Query of the batch",
      "day (not in results)",
    ]);
    expect(options(wrapper, "chart-measure-key")).toEqual([
      "load (not in results)",
    ]);
  });

  it("hides the grouping for a stat tile", () => {
    const wrapper = mountSettings({ ...countByType, kind: "stat" });
    expect(wrapper.find('[data-testid="chart-group-by"]').exists()).toBe(false);
  });
});
