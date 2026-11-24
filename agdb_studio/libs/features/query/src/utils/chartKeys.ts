/**
 * The mark a key moves to in a chart read with the arrow keys: one tab stop
 * for the chart, then Left/Up and Right/Down step through its marks and
 * Home/End jump to either end. Undefined for other keys.
 */
export const stepMark = (
  key: string,
  current: number | undefined,
  count: number,
): number | undefined => {
  if (!count) return undefined;
  const last = count - 1;
  switch (key) {
    case "ArrowRight":
    case "ArrowDown":
      return Math.min(last, (current ?? -1) + 1);
    case "ArrowLeft":
    case "ArrowUp":
      return Math.max(0, (current ?? count) - 1);
    case "Home":
      return 0;
    case "End":
      return last;
    default:
      return undefined;
  }
};
