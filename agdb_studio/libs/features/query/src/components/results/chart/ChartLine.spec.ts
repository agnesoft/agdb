import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ChartLine from "./ChartLine.vue";
import type { ChartAxis, ChartPoint } from "../../../utils/chartData";

const series = (labels: string[], values: number[]): ChartPoint[] =>
  labels.map((label, i) => ({ key: label, label, value: values[i] ?? 0 }));

const mountLine = (points: ChartPoint[], axis: ChartAxis = "category") =>
  mount(ChartLine, {
    props: {
      xs: points,
      series: [
        {
          key: "value",
          label: "Count by day",
          color: "var(--graph-palette-1)",
          values: points.map((p) => p.value),
        },
      ],
      axis,
      width: 400,
      height: 200,
      label: "Count by day",
    },
  });

const tooltip = (wrapper: ReturnType<typeof mountLine>) =>
  wrapper.find('[data-testid="chart-tooltip"]');

describe("ChartLine", () => {
  it("draws a line with markers and labels the last and highest points", () => {
    const wrapper = mountLine(series(["a", "b", "c"], [5, 9, 7]));
    expect(wrapper.find(".line").attributes("d")).toMatch(/^M.* L.* L/);
    expect(wrapper.findAll(".marker")).toHaveLength(3);
    expect(wrapper.findAll(".value").map((v) => v.text())).toEqual(["7", "9"]);
    expect(wrapper.find("svg").attributes("aria-label")).toContain(
      "arrow keys",
    );
    // The end labels stay within the plot.
    expect(
      wrapper
        .findAll(".grid text[text-anchor]")
        .map((t) => t.attributes("text-anchor"))
        .slice(-3),
    ).toEqual(["start", "middle", "end"]);
  });

  it("places numbers and dates by value along X", () => {
    const xOf = (wrapper: ReturnType<typeof mountLine>) =>
      wrapper.findAll(".marker").map((m) => Number(m.attributes("cx")));
    const numbers = xOf(
      mountLine(series(["1", "2", "10"], [1, 2, 3]), "number"),
    );
    // 2 sits much nearer 1 than 10.
    expect(numbers[1]! - numbers[0]!).toBeLessThan(numbers[2]! - numbers[1]!);
    const dates = xOf(
      mountLine(
        series(["2024-01-01", "2024-01-02", "2024-02-01"], [1, 2, 3]),
        "date",
      ),
    );
    expect(dates[1]! - dates[0]!).toBeLessThan(dates[2]! - dates[1]!);
    // One point sits in the middle.
    expect(xOf(mountLine(series(["x"], [4]), "number"))).toHaveLength(1);
  });

  it("shows markers only on hover when there are many points", async () => {
    const labels = Array.from({ length: 40 }, (_, i) => `${i}`);
    const wrapper = mountLine(series(labels, labels.map(Number)), "number");
    expect(wrapper.findAll(".marker")).toHaveLength(0);
    // Not every X label fits.
    expect(wrapper.findAll(".grid text").length).toBeLessThan(40 + 5);
    await wrapper.find("svg").trigger("mousemove", { clientX: 400 });
    expect(wrapper.findAll(".marker")).toHaveLength(1);
    expect(tooltip(wrapper).text()).toContain("39");
  });

  it("follows the pointer with a crosshair and the nearest point", async () => {
    const wrapper = mountLine(series(["a", "b", "c"], [5, 9, 7]));
    const svg = wrapper.find("svg");
    await svg.trigger("mousemove", { clientX: 0 });
    expect(tooltip(wrapper).text()).toContain("a");
    expect(wrapper.find(".crosshair").exists()).toBe(true);
    await svg.trigger("mousemove", { clientX: 1000 });
    expect(tooltip(wrapper).text()).toContain("c");
    await svg.trigger("mouseleave");
    expect(tooltip(wrapper).exists()).toBe(false);
  });

  it("reads the points with the keyboard", async () => {
    const wrapper = mountLine(series(["a", "b", "c"], [5, 9, 7]));
    const svg = wrapper.find('[data-testid="chart-line-plot"]');
    const key = async (k: string) => {
      await svg.trigger("keydown", { key: k });
      return tooltip(wrapper).find(".label").text();
    };
    const announced = () =>
      wrapper.find('[data-testid="chart-line-announcement"]').text();
    expect(announced()).toBe("");
    expect(await key("ArrowRight")).toBe("a");
    expect(announced()).toBe("a: 5");
    expect(await key("ArrowRight")).toBe("b");
    expect(await key("End")).toBe("c");
    expect(await key("ArrowRight")).toBe("c");
    expect(await key("ArrowLeft")).toBe("b");
    expect(await key("Home")).toBe("a");
    expect(await key("ArrowLeft")).toBe("a");
    // Other keys change nothing.
    expect(await key("x")).toBe("a");
    await svg.trigger("blur");
    expect(tooltip(wrapper).exists()).toBe(false);
    expect(await key("ArrowLeft")).toBe("c");
  });

  describe("with several lines", () => {
    const xs = ["a", "b", "c"].map((label) => ({ key: label, label }));
    const line = (n: number, values: (number | undefined)[]) => ({
      key: `query-${n}`,
      label: `#${n + 1} select.search.from.where`,
      color: `var(--graph-palette-${n + 1})`,
      values,
    });
    const mountLines = (series: ReturnType<typeof line>[]) =>
      mount(ChartLine, {
        props: {
          xs,
          series,
          axis: "category",
          width: 400,
          height: 200,
          label: "Count by day, per query",
        },
      });

    it("draws each line in its colour with a legend and names at the ends", () => {
      const wrapper = mountLines([line(0, [1, 4, 2]), line(1, [3, 3, 2])]);
      const paths = wrapper.findAll(".line");
      expect(paths).toHaveLength(2);
      expect(paths[1]!.attributes("style")).toContain("--graph-palette-2");
      expect(
        wrapper.findAll(".legend li").map((li) => li.attributes("title")),
      ).toEqual(["#1 select.search.from.where", "#2 select.search.from.where"]);
      // Names, not values; both end at 2, so one is nudged below the other.
      expect(wrapper.findAll(".value")).toHaveLength(0);
      const ends = wrapper.findAll(".end-label");
      expect(ends.map((e) => e.text())).toEqual([
        "#1 select.sea…",
        "#2 select.sea…",
      ]);
      expect(Number(ends[1]!.attributes("y"))).toBeGreaterThan(
        Number(ends[0]!.attributes("y")),
      );
      expect(wrapper.find("svg").attributes("aria-label")).toContain("2 lines");
    });

    it("joins a line's own points over X only other lines have", () => {
      const labels = Array.from({ length: 40 }, (_, i) => `${i}`);
      const many = labels.map((label) => ({ key: label, label }));
      const values = labels.map((_, i) => (i === 20 || i > 30 ? undefined : i));
      values[30] = 30;
      values[32] = 5;
      const wrapper = mount(ChartLine, {
        props: {
          xs: many,
          series: [
            line(0, values),
            line(
              1,
              labels.map(() => 1),
            ),
          ],
          axis: "number",
          width: 400,
          height: 200,
          label: "Count",
        },
      });
      const d = wrapper.find(".line").attributes("d")!;
      // One line through its 31 points, over the X it has no value at.
      expect(d.match(/M/g)).toHaveLength(1);
      expect(d.match(/L/g)).toHaveLength(30);
      // Too many points for markers.
      expect(wrapper.findAll(".marker")).toHaveLength(0);
      // A line of one point is a marker.
      const one = mount(ChartLine, {
        props: {
          xs: many,
          series: [
            line(
              0,
              labels.map((_, i) => (i === 3 ? 7 : undefined)),
            ),
            line(
              1,
              labels.map(() => 1),
            ),
          ],
          axis: "number",
          width: 400,
          height: 200,
          label: "Count",
        },
      });
      expect(one.findAll(".marker")).toHaveLength(1);
    });

    it("lists every line's value at the hovered X", async () => {
      const wrapper = mountLines([
        line(0, [1, undefined, 2]),
        line(1, [3, 3, 2]),
      ]);
      const svg = wrapper.find("svg");
      await svg.trigger("mousemove", { clientX: 200 });
      const rows = wrapper
        .findAll('[data-testid="chart-tooltip"] li')
        .map((li) => li.text());
      expect(rows).toEqual([
        "#1 select.search.from.where–",
        "#2 select.search.from.where3",
      ]);
      expect(wrapper.find('[data-testid="chart-tooltip"] .label').text()).toBe(
        "b",
      );
      // Read out too, for screen readers.
      expect(
        wrapper.find('[data-testid="chart-line-announcement"]').text(),
      ).toBe(
        "b: #1 select.search.from.where: –, #2 select.search.from.where: 3",
      );
    });

    it("highlights a line from its legend entry", async () => {
      const wrapper = mountLines([line(0, [1, 4, 2]), line(1, [3, 3, 2])]);
      const entry = wrapper.findAll(".legend li")[1]!;
      await entry.trigger("mouseenter");
      expect(
        wrapper.find('[data-testid="chart-line-series-0"]').classes(),
      ).toContain("dim");
      expect(wrapper.findAll(".legend li.dim")).toHaveLength(1);
      await entry.trigger("mouseleave");
      expect(wrapper.findAll(".series.dim")).toHaveLength(0);
    });

    it("names only the lines that reach the end, within the plot", () => {
      // Flat at 1 (as when counting unique values): all three ends meet.
      const wrapper = mountLines([
        line(0, [1, 1, 1]),
        line(1, [1, 1, 1]),
        line(2, [1, 1, 1]),
      ]);
      const ys = wrapper
        .findAll(".end-label")
        .map((e) => Number(e.attributes("y")));
      expect(ys).toHaveLength(3);
      // Apart, and none below the plot (onto the X labels).
      expect(ys[1]! - ys[0]!).toBeGreaterThanOrEqual(13);
      const plotBottom = Math.max(
        ...wrapper.findAll(".grid line").map((l) => Number(l.attributes("y1"))),
      );
      expect(Math.max(...ys)).toBeLessThanOrEqual(plotBottom + 4);
      // A flat line sits mid-plot, with ticks either side.
      expect(wrapper.findAll(".grid text").map((t) => t.text())).toEqual(
        expect.arrayContaining(["0", "1", "2"]),
      );
      // A line that ends early gets no name (the legend has it).
      const early = mountLines([
        line(0, [1, 2, 3]),
        line(1, [2, 1, undefined]),
      ]);
      expect(early.findAll(".end-label").map((e) => e.text())).toEqual([
        "#1 select.sea…",
      ]);
    });

    it("keeps X labels apart where numbers crowd together", () => {
      const prices = [12, 18, 32, 40, 80, 90, 120, 250, 320, 1400];
      const wrapper = mount(ChartLine, {
        props: {
          xs: prices.map((p) => ({ key: `${p}`, label: `${p}` })),
          series: [
            line(
              0,
              prices.map(() => 1),
            ),
          ],
          axis: "number",
          width: 400,
          height: 200,
          label: "Count by price",
        },
      });
      const labels = wrapper.findAll(".x-label");
      const texts = labels.map((t) => t.text());
      // The first and the last stay; crowded ones in between give way.
      expect(texts[0]).toBe("12");
      expect(texts.at(-1)).toBe("1400");
      expect(texts.length).toBeLessThan(prices.length);
      const xs = labels.map((t) => Number(t.attributes("x")));
      xs.slice(1).forEach((x, i) =>
        expect(x - xs[i]!).toBeGreaterThanOrEqual(25),
      );
    });

    it("names many lines in the legend only", () => {
      const wrapper = mountLines(
        [0, 1, 2, 3, 4].map((n) => line(n, [n, n + 1, n])),
      );
      expect(wrapper.findAll(".legend li")).toHaveLength(5);
      expect(wrapper.findAll(".end-label")).toHaveLength(0);
      // A line without values draws nothing, but stays in the legend.
      const empty = mountLines([
        line(0, [1, 2, 3]),
        line(1, [undefined, undefined, undefined]),
      ]);
      expect(empty.findAll(".end-label")).toHaveLength(1);
      expect(empty.findAll(".legend li")).toHaveLength(2);
    });
  });
});
