/** The smallest number; no spread, so any number of values is safe. */
export const minOf = (values: readonly number[]): number =>
  values.reduce((a, b) => (b < a ? b : a), Infinity);

/** The largest number; no spread, so any number of values is safe. */
export const maxOf = (values: readonly number[]): number =>
  values.reduce((a, b) => (b > a ? b : a), -Infinity);

/** A value axis: its domain and round tick values within it. */
export type ValueScale = { min: number; max: number; ticks: number[] };

const niceStep = (span: number, count: number): number => {
  const raw = span / Math.max(1, count);
  const magnitude = 10 ** Math.floor(Math.log10(raw));
  const residual = raw / magnitude;
  // Rounds to 1, 2, 5 or 10 at the geometric midpoints, so the axis ends near the data.
  const factor =
    residual >= 7.07 ? 10 : residual >= 3.16 ? 5 : residual >= 1.41 ? 2 : 1;
  return factor * magnitude;
};

/**
 * A value scale covering `values`, with about `count` round ticks. Bars start
 * at zero, so it is included unless `includeZero` is off (e.g. for lines).
 */
export const valueScale = (
  values: number[],
  count = 4,
  includeZero = true,
): ValueScale => {
  const base = includeZero ? [0] : [];
  let lo = minOf([...base, ...values]);
  let hi = maxOf([...base, ...values]);
  // One value: a unit either side, so a flat line sits in the middle, not
  // on the edge (an axis from zero of only zeros goes up). The unit is one
  // the float spacing at the value allows: at 1e17, adding 1 changes nothing.
  const spacing = Math.abs(lo) * Number.EPSILON * 64;
  if (hi - lo <= spacing) {
    const unit = Math.max(1, spacing);
    if (includeZero) hi = lo + unit;
    else {
      lo -= unit;
      hi += unit;
    }
  }
  // Whole numbers (e.g. counts) get whole ticks: no 0.2 between 0 and 1.
  const whole = values.every(Number.isInteger);
  const step = Math.max(niceStep(hi - lo, count), whole ? 1 : 0);
  lo = Math.floor(lo / step) * step;
  hi = Math.ceil(hi / step) * step;
  const ticks: number[] = [];
  // Rounded, so float steps (e.g. 0.1) give clean tick values.
  // Capped too, should rounding ever stall the steps.
  for (let t = lo; t <= hi + step / 2 && ticks.length <= 100; t += step) {
    ticks.push(Number(t.toPrecision(12)));
  }
  return { min: lo, max: hi, ticks };
};

const compact = new Intl.NumberFormat(undefined, {
  notation: "compact",
  maximumFractionDigits: 1,
});
const full = new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 });

/** Short form for axes and labels, e.g. 1.2K. */
export const formatCompact = (value: number): string => compact.format(value);

/** Exact form for tooltips and stat tiles, e.g. 1,234.57. */
export const formatValue = (value: number): string => full.format(value);

/** Shortens a label to `max` characters with an ellipsis. */
export const truncate = (label: string, max: number): string =>
  label.length > max ? `${label.slice(0, Math.max(1, max - 1))}…` : label;

/** Approximate width of one character of chart text (11px), for layout. */
export const CHAR_WIDTH = 6.5;

/** A series colour: the graph palette's `slot`, from the theme. */
export const paletteColor = (slot: number): string =>
  `var(--graph-palette-${slot + 1})`;

/** The colour of "Other" and anything no series colour applies to. */
export const OTHER_COLOR = "var(--graph-uncolored-color)";

/**
 * An SVG path of a bar from `base` to `end` along its value axis, with only
 * the data end rounded, so the bar stays anchored to the baseline.
 * `vertical` bars grow along y; their cross axis is x (and the reverse).
 */
export const barPath = (
  vertical: boolean,
  cross: number,
  thickness: number,
  base: number,
  end: number,
  radius = 4,
): string => {
  const length = Math.abs(end - base);
  const r = Math.min(radius, thickness / 2, length);
  const dir = end < base ? -1 : 1;
  const a = cross;
  const b = cross + thickness;
  // Points along the value axis (v) and the cross axis (c), as x/y per orientation.
  const p = (v: number, c: number) => (vertical ? `${c},${v}` : `${v},${c}`);
  const sweep = vertical ? (dir < 0 ? 1 : 0) : dir > 0 ? 1 : 0;
  return [
    `M${p(base, a)}`,
    `L${p(end - dir * r, a)}`,
    `A${r},${r} 0 0 ${sweep} ${p(end, a + r)}`,
    `L${p(end, b - r)}`,
    `A${r},${r} 0 0 ${sweep} ${p(end - dir * r, b)}`,
    `L${p(base, b)}`,
    "Z",
  ].join(" ");
};
