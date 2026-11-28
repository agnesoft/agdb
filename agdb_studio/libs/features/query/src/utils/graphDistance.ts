import { linkEndpointId } from "./graphStyle";

type Link = { source: unknown; target: unknown };

/**
 * Hops from `startId` to every node it reaches through `links`, following
 * edges either way, as the graph shows them. Nodes it doesn't reach are left out.
 */
export const distanceFrom = (
  startId: number,
  links: readonly Link[],
): Map<number, number> => {
  const neighbours = new Map<number, number[]>();
  const connect = (a: number, b: number) => {
    const list = neighbours.get(a) ?? [];
    list.push(b);
    neighbours.set(a, list);
  };
  for (const link of links) {
    const source = linkEndpointId(link.source);
    const target = linkEndpointId(link.target);
    connect(source, target);
    connect(target, source);
  }
  const hops = new Map<number, number>([[startId, 0]]);
  const queue = [startId];
  for (let i = 0; i < queue.length; i++) {
    const id = queue[i]!;
    const next = hops.get(id)! + 1;
    for (const neighbour of neighbours.get(id) ?? []) {
      if (hops.has(neighbour)) continue;
      hops.set(neighbour, next);
      queue.push(neighbour);
    }
  }
  return hops;
};

type QueryIdValue = { Id: number } | { Alias: string };

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;

const isQueryId = (value: unknown): value is QueryIdValue =>
  isRecord(value) &&
  ((typeof value.Id === "number" && value.Id !== 0) ||
    typeof value.Alias === "string");

/** The first search in `value` (searches nest in selects), depth first. */
const findSearch = (
  value: unknown,
  seen: Set<object>,
): Record<string, unknown> | undefined => {
  if (!isRecord(value) || seen.has(value)) return undefined;
  seen.add(value);
  if ("algorithm" in value && "origin" in value) return value;
  for (const child of Object.values(value)) {
    const found = findSearch(child, seen);
    if (found) return found;
  }
  return undefined;
};

/**
 * The node the first search of `queries` started from: its origin, or its
 * destination for a search that only has one (searching backwards). An alias
 * is looked up with `aliasId`, only when there is one.
 */
export const searchStart = (
  queries: readonly unknown[],
  aliasId: (alias: string) => number | undefined,
): number | undefined => {
  const seen = new Set<object>();
  for (const query of queries) {
    const search = findSearch(query, seen);
    if (!search) continue;
    const start = [search.origin, search.destination].find(isQueryId);
    if (!start) return undefined;
    return "Id" in start ? start.Id : aliasId(start.Alias);
  }
  return undefined;
};
