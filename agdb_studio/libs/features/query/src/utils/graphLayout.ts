/** Layout fields that 3d-force-graph stores on each node object. */
const LAYOUT_KEYS = [
  "x",
  "y",
  "z",
  "vx",
  "vy",
  "vz",
  "fx",
  "fy",
  "fz",
] as const;

/**
 * Copies the layout of nodes that were already shown onto the new node
 * objects (matched by id), so updating the data does not reset the layout.
 * Returns whether any node is new, i.e. whether the layout needs to settle.
 */
export const carryOverLayout = (
  previous: readonly { id: number }[],
  next: readonly { id: number }[],
): boolean => {
  const byId = new Map(previous.map((node) => [node.id, node]));
  let added = false;
  for (const node of next) {
    const old = byId.get(node.id);
    if (!old) {
      added = true;
      continue;
    }
    for (const key of LAYOUT_KEYS) {
      const value: unknown = Reflect.get(old, key);
      if (typeof value === "number") Reflect.set(node, key, value);
    }
  }
  return added;
};
