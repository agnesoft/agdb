import type { DbKeyValue, QueryResult, QueryType } from "@agnesoft/agdb_api";
import { reactive } from "vue";
import type { ElementValuesState, GraphElement } from "./types";
import {
  selectAllAliasesQuery,
  selectElementQuery,
  selectExistingElementsQuery,
} from "../utils/graphActionQueries";
import { dbValueToString, toDisplayValues } from "../utils/graphDataUtils";

/** Elements whose stored values are loaded right after a run. */
export const PREFETCH_LIMIT = 1000;
/** Ids per `select().ids()` query; all chunks go in one request. */
const CHUNK_SIZE = 500;

type Entry = { state: ElementValuesState; values?: DbKeyValue[] };

export type ExecQueries = (
  queries: QueryType[],
) => Promise<QueryResult[] | undefined>;

const keyId = (kv: DbKeyValue) => JSON.stringify(kv.key);

/**
 * Cache of elements' stored values. Queries often return only some values
 * (or computed ones such as `edge_count`), but the details panel, tooltips
 * and the editor need everything stored on the element.
 */
/** The server's status for a query that failed, e.g. on an id that does not exist. */
const QUERY_ERROR_STATUS = 470;

const isQueryError = (error: unknown): boolean => {
  if (typeof error !== "object" || error === null || !("response" in error)) {
    return false;
  }
  const { response } = error;
  return (
    typeof response === "object" &&
    response !== null &&
    "status" in response &&
    response.status === QUERY_ERROR_STATUS
  );
};

export const useElementValues = (exec: ExecQueries) => {
  const entries = reactive(new Map<number, Entry>());
  const aliases = reactive(new Map<number, string>());
  /** Bumped on `clear` so responses of an outdated load are dropped. */
  let generation = 0;

  /**
   * Loads stored values of the given ids that are not cached (or loading).
   * With `retry`, ids whose last load failed are loaded again.
   */
  const load = async (
    ids: number[],
    { retry = false }: { retry?: boolean } = {},
  ): Promise<void> => {
    const missing = [...new Set(ids)].filter((id) => {
      if (id === 0) return false; // id 0 is database metadata, not an element.
      const entry = entries.get(id);
      return !entry || (retry && entry.state === "error");
    });
    if (missing.length === 0) return;

    const startedIn = generation;
    for (const id of missing) entries.set(id, { state: "loading" });

    const store = (results: QueryResult[] | undefined) => {
      for (const el of (results ?? []).flatMap((r) => r.elements)) {
        entries.set(el.id, { state: "loaded", values: el.values });
      }
    };

    // One request per chunk: agdb rejects a whole query if any id no longer
    // exists, so a rejected chunk is asked again for the existing ids only.
    const fetchValues = async (chunk: number[]): Promise<void> => {
      try {
        const results = await exec([selectElementQuery(chunk)]);
        if (startedIn === generation) store(results);
      } catch (error) {
        // Network or auth errors would fail the fallback too.
        if (!isQueryError(error) || startedIn !== generation) return;
        try {
          const results = await exec([selectExistingElementsQuery(chunk)]);
          if (startedIn === generation) store(results);
        } catch {
          // Marked as failed below.
        }
      }
    };
    const chunks = [];
    for (let i = 0; i < missing.length; i += CHUNK_SIZE) {
      chunks.push(missing.slice(i, i + CHUNK_SIZE));
    }
    await Promise.all(chunks.map(fetchValues));

    if (startedIn !== generation) return;
    // Anything not loaded (failed, or not returned) is marked as failed.
    for (const id of missing) {
      if (entries.get(id)?.state === "loading") {
        entries.set(id, { state: "error" });
      }
    }
  };

  /**
   * Loads all aliases, replacing the cached ones. Failing to read them only
   * means none are shown, so errors are ignored.
   */
  const loadAliases = async (): Promise<void> => {
    const startedIn = generation;
    try {
      const results = await exec([selectAllAliasesQuery()]);
      if (startedIn !== generation) return;
      aliases.clear();
      for (const el of (results ?? []).flatMap((r) => r.elements)) {
        const alias = el.values[0]?.value;
        if (alias) aliases.set(el.id, dbValueToString(alias));
      }
    } catch {
      // No aliases shown.
    }
  };

  const aliasOf = (id: number): string | undefined => aliases.get(id);

  /** Stores values known to be current (e.g. re-selected after a save). */
  const set = (id: number, values: DbKeyValue[]) => {
    entries.set(id, { state: "loaded", values });
  };

  const remove = (id: number) => {
    entries.delete(id);
    aliases.delete(id);
  };

  /** Forgets everything, e.g. when a new run may have changed values. */
  const clear = () => {
    generation++;
    entries.clear();
  };

  /**
   * Returns the element with its stored values once loaded. Values the query
   * returned under keys that are not stored are kept in `queryOnlyValues`.
   */
  const describe = <T extends GraphElement>(element: T): T => {
    const entry = entries.get(element.id);
    const alias = aliases.get(element.id);
    const withAlias = alias === undefined ? element : { ...element, alias };
    if (!entry) return withAlias;
    if (entry.state !== "loaded" || !entry.values) {
      return { ...withAlias, valuesState: entry.state };
    }
    const stored = entry.values;
    const storedKeys = new Set(stored.map(keyId));
    return {
      ...withAlias,
      rawValues: stored,
      values: toDisplayValues(stored),
      valuesState: "loaded",
      queryOnlyValues: element.rawValues.filter(
        (kv) => !storedKeys.has(keyId(kv)),
      ),
    };
  };

  return { load, loadAliases, aliasOf, set, remove, clear, describe };
};
