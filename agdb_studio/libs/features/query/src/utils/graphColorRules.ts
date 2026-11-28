import type { GraphElement, GraphLink } from "../composables/types";
import { distanceFrom } from "./graphDistance";
import {
  DEFAULT_RAMP,
  RAMP_NAMES,
  rampColor,
  type RampName,
} from "./graphStyle";

/**
 * A colour picked from the graph palette, by slot (0-based) so it follows the
 * light/dark theme instead of being a fixed hex value.
 */
export type PaletteSlot = number;

/** Colours elements whose value under `key` matches. */
export type PropertyColorRule = {
  id: string;
  key: string;
  /** "eq": the value equals `value` (as displayed); "exists": the key is present. */
  op: "eq" | "exists";
  value?: string;
  color: PaletteSlot;
};

/** Gives each distinct value under `key` its own palette colour. */
export type PaletteColorRule = {
  id: string;
  key: string;
};

/** Colours elements along a gradient by a numeric value under `key`, low to high. */
export type GradientColorRule = {
  id: string;
  key: string;
  /** The gradient's scale; teal when not set. */
  scale?: RampName;
};

/**
 * Colours nodes along the gradient by their hops from a start node: the
 * search start of the last run, or a node by id.
 */
export type DistanceColorRule = {
  id: string;
  from: number | "start";
  /** The gradient's scale; teal when not set. */
  scale?: RampName;
};

export type ColorRule =
  | ({ kind: "property" } & PropertyColorRule)
  | ({ kind: "palette" } & PaletteColorRule)
  | ({ kind: "gradient" } & GradientColorRule)
  | ({ kind: "distance" } & DistanceColorRule);

/** A colour: a palette slot, or a position on a gradient scale (0 low, 1 high). */
export type ColorRef =
  { slot: PaletteSlot } | { ramp: number; scale: RampName };

/** The scale of a gradient or distance rule. */
export const ruleScale = (rule: { scale?: RampName }): RampName =>
  rule.scale ?? DEFAULT_RAMP;

/** Distances shown apart; this many hops and more share the last step. */
export const MAX_DISTANCE = 6;

/** The start strongest, the farthest step reached faintest, spread evenly. */
const distanceRamp = (step: number, farthest: number): number =>
  farthest ? 1 - step / farthest : 1;

const DECIMAL = /^\s*[-+]?(\d+\.?\d*|\.\d+)(e[-+]?\d+)?\s*$/i;

/** A value as a decimal number, if it reads as one. */
export const numericValue = (value: string): number | undefined => {
  // Decimal only: Number() also reads "0x1f" and "0b1", e.g. string ids.
  if (!DECIMAL.test(value)) return undefined;
  const number = Number(value);
  return Number.isFinite(number) ? number : undefined;
};

/** A gradient end, without float noise such as 0.30000000000000004. */
const formatScaleEnd = (value: number) =>
  value.toLocaleString(undefined, { maximumFractionDigits: 3 });

/** Colouring of a query's graph; kept with the query. */
export type ColorConfig = {
  /** Applied in order; the first rule that matches wins. */
  rules: ColorRule[];
  /** Colour of the elements returned by a builder row, by row id. */
  rows: Record<string, PaletteSlot>;
  /** Colours set by hand on single elements, by element id. */
  elements: Record<number, PaletteSlot>;
};

/**
 * A copy sharing nothing with `config`. Rules are flat, so copying each is
 * enough, and it works on reactive proxies, which structuredClone rejects.
 */
export const copyColorConfig = (config: ColorConfig): ColorConfig => ({
  rules: config.rules.map((rule) => ({ ...rule })),
  rows: { ...config.rows },
  elements: { ...config.elements },
});

export const emptyColorConfig = (): ColorConfig => ({
  rules: [],
  rows: {},
  elements: {},
});

export const hasColoring = (config: ColorConfig | undefined): boolean =>
  !!config &&
  (config.rules.length > 0 ||
    Object.keys(config.rows).length > 0 ||
    Object.keys(config.elements).length > 0);

const valueOf = (element: GraphElement, key: string): string | undefined =>
  element.values.find((v) => v.key === key)?.value;

/** Where a colour comes from, in the order they override each other. */
export type ColorSource = "element" | "property" | "query";

/** A gradient in the legend: its scale and what its ends stand for. */
export type LegendScale = {
  ramp: RampName;
  start: string;
  end: string;
  descending?: boolean;
};

/** One line of the colour legend: a colour, what it means and how many it colours. */
export type LegendEntry = {
  label: string;
  color: ColorRef;
  source: ColorSource;
  count: number;
  /**
   * A gradient: what its ends stand for, low to high; `descending` when the
   * low end is the strong colour (distance: the start is strongest).
   */
  scale?: LegendScale;
};

export type ColorResolver = {
  /** The colour of an element, or undefined for the default colour. */
  colorOf: (element: GraphElement) => ColorRef | undefined;
  /** Only the colours that colour some of the elements, grouped by source. */
  legend: LegendEntry[];
};

export type ColorOptions = {
  row?: (rowId: string) => string;
  element?: (id: number) => string;
  /** The node the search of the last run started from. */
  startId?: number;
};

type Match = { key: string; color: ColorRef };

/** Edges have endpoints; nodes have a `source` too, but it says where they came from. */
const isLink = (element: GraphElement): element is GraphLink =>
  "target" in element;

/** A distance rule's start node, if known. */
export const distanceStart = (
  rule: DistanceColorRule,
  startId: number | undefined,
): number | undefined => (rule.from === "start" ? startId : rule.from);

/**
 * Builds the colouring of the shown elements. Precedence: a colour set by
 * hand, then the first matching rule, then the colour of a row that returned
 * the element. Palette rules give distinct values the palette's colours in
 * order of first appearance; values beyond the palette keep the default
 * colour (colours are never reused for different values). Gradient rules
 * spread a numeric value from its lowest to its highest shown value; distance
 * rules colour nodes by their hops from a start node, the start strongest
 * and the farthest shown faintest.
 * `rowIdsByResult` maps result indexes to the rows that produced them.
 */
export const buildColorResolver = (
  config: ColorConfig | undefined,
  elements: readonly GraphElement[],
  rowIdsByResult: readonly (string | undefined)[],
  paletteSize: number,
  options: ColorOptions = {},
): ColorResolver => {
  if (!config || !hasColoring(config)) {
    return { colorOf: () => undefined, legend: [] };
  }
  const rowLabel = options.row ?? ((rowId: string) => rowId);
  const elementLabel = options.element ?? ((id: number) => `element ${id}`);

  // Legend entries in display order, keyed by what produced them.
  const entries = new Map<string, Omit<LegendEntry, "count">>();
  for (const [id, slot] of Object.entries(config.elements)) {
    entries.set(`e:${id}`, {
      label: elementLabel(Number(id)),
      color: { slot },
      source: "element",
    });
  }
  const paletteSlots = new Map<string, Map<string, PaletteSlot>>();
  const scales = new Map<string, { min: number; max: number }>();
  const distances = new Map<
    string,
    { hops: Map<number, number>; farthest: number }
  >();
  const links = elements.filter(isLink);
  for (const rule of config.rules) {
    if (rule.kind === "gradient") {
      const numbers = elements.flatMap((element) => {
        const value = valueOf(element, rule.key);
        const number = value === undefined ? undefined : numericValue(value);
        return number === undefined ? [] : [number];
      });
      if (!numbers.length) continue;
      // A loop, not Math.min(...), which overflows the stack on big results.
      const scale = { min: numbers[0]!, max: numbers[0]! };
      for (const number of numbers) {
        if (number < scale.min) scale.min = number;
        if (number > scale.max) scale.max = number;
      }
      scales.set(rule.id, scale);
      entries.set(`r:${rule.id}`, {
        label: rule.key,
        // Stands for the scale where one colour is needed; the legend draws `scale`.
        color: { ramp: 0.5, scale: ruleScale(rule) },
        source: "property",
        scale: {
          ramp: ruleScale(rule),
          start: formatScaleEnd(scale.min),
          end: formatScaleEnd(scale.max),
        },
      });
      continue;
    }
    if (rule.kind === "distance") {
      const start = distanceStart(rule, options.startId);
      if (start === undefined) continue;
      if (!elements.some((e) => e.id === start && !isLink(e))) continue;
      const hops = distanceFrom(start, links);
      // The whole gradient over the distances shown, so near steps stay apart.
      let reached = 0;
      for (const step of hops.values()) reached = Math.max(reached, step);
      const farthest = Math.min(reached, MAX_DISTANCE);
      distances.set(rule.id, { hops, farthest });
      entries.set(`r:${rule.id}`, {
        label: "distance",
        // Stands for the scale where one colour is needed; the legend draws `scale`.
        color: { ramp: 1, scale: ruleScale(rule) },
        source: "property",
        scale: {
          ramp: ruleScale(rule),
          start: "0",
          end: `${farthest}${reached > MAX_DISTANCE ? "+" : ""}`,
          descending: true,
        },
      });
      continue;
    }
    if (rule.kind === "property") {
      entries.set(`r:${rule.id}`, {
        label:
          rule.op === "exists"
            ? `has ${rule.key}`
            : `${rule.key} = ${rule.value ?? ""}`,
        color: { slot: rule.color },
        source: "property",
      });
      continue;
    }
    const slots = new Map<string, PaletteSlot>();
    for (const element of elements) {
      const value = valueOf(element, rule.key);
      if (value === undefined || slots.has(value)) continue;
      if (slots.size >= paletteSize) break;
      const slot = slots.size;
      slots.set(value, slot);
      entries.set(`r:${rule.id}:${value}`, {
        label: `${rule.key} = ${value}`,
        color: { slot },
        source: "property",
      });
    }
    paletteSlots.set(rule.id, slots);
  }
  for (const [rowId, slot] of Object.entries(config.rows)) {
    entries.set(`q:${rowId}`, {
      label: rowLabel(rowId),
      color: { slot },
      source: "query",
    });
  }

  const ruleMatch = (
    rule: ColorRule,
    element: GraphElement,
  ): Match | undefined => {
    if (rule.kind === "distance") {
      const distance = distances.get(rule.id);
      const hops = isLink(element) ? undefined : distance?.hops.get(element.id);
      if (!distance || hops === undefined) return undefined;
      const step = Math.min(hops, MAX_DISTANCE);
      return {
        key: `r:${rule.id}`,
        color: {
          ramp: distanceRamp(step, distance.farthest),
          scale: ruleScale(rule),
        },
      };
    }
    const value = valueOf(element, rule.key);
    if (value === undefined) return undefined;
    if (rule.kind === "gradient") {
      const scale = scales.get(rule.id);
      const number = numericValue(value);
      if (!scale || number === undefined) return undefined;
      const span = scale.max - scale.min;
      return {
        key: `r:${rule.id}`,
        color: {
          ramp: span ? (number - scale.min) / span : 0.5,
          scale: ruleScale(rule),
        },
      };
    }
    if (rule.kind === "palette") {
      const slot = paletteSlots.get(rule.id)?.get(value);
      return slot === undefined
        ? undefined
        : { key: `r:${rule.id}:${value}`, color: { slot } };
    }
    if (rule.op === "exists" || value === rule.value) {
      return { key: `r:${rule.id}`, color: { slot: rule.color } };
    }
    return undefined;
  };

  const match = (element: GraphElement): Match | undefined => {
    const manual = config.elements[element.id];
    if (manual !== undefined) {
      return { key: `e:${element.id}`, color: { slot: manual } };
    }
    for (const rule of config.rules) {
      const found = ruleMatch(rule, element);
      if (found) return found;
    }
    for (const index of element.resultIndexes ?? []) {
      const rowId = rowIdsByResult[index];
      const slot = rowId === undefined ? undefined : config.rows[rowId];
      if (slot !== undefined) return { key: `q:${rowId}`, color: { slot } };
    }
    return undefined;
  };

  const counts = new Map<string, number>();
  for (const element of elements) {
    const found = match(element);
    if (found) counts.set(found.key, (counts.get(found.key) ?? 0) + 1);
  }
  const legend = [...entries.entries()].flatMap(([key, entry]) => {
    const count = counts.get(key) ?? 0;
    return count ? [{ ...entry, count }] : [];
  });

  return { colorOf: (element) => match(element)?.color, legend };
};

/** The colour of a palette slot, if the palette has it. */
export const slotColor = (
  palette: readonly string[],
  slot: PaletteSlot,
): string | undefined => palette[slot];

/** The colour a `ColorRef` stands for in the theme, if the palette has it. */
export const resolveColor = (
  palette: readonly string[],
  ramps: Readonly<Record<RampName, readonly string[]>>,
  color: ColorRef,
): string | undefined =>
  "slot" in color
    ? slotColor(palette, color.slot)
    : rampColor(ramps[color.scale], color.ramp);

/**
 * A scale for a new gradient or distance rule: the first one `rules` don't
 * use yet, so two gradients don't look alike, or the default when all are.
 */
export const unusedScale = (rules: readonly ColorRule[]): RampName => {
  const used = new Set(
    rules.flatMap((rule) =>
      rule.kind === "gradient" || rule.kind === "distance"
        ? [ruleScale(rule)]
        : [],
    ),
  );
  return RAMP_NAMES.find((name) => !used.has(name)) ?? DEFAULT_RAMP;
};
