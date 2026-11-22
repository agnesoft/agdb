import type { QueryResult, QueryType } from "@agnesoft/agdb_api";
import { checkClient, client } from "@agdb-studio/api/src/api";
import { useRoute } from "vue-router";
import {
  hasInjectionContext,
  inject,
  provide,
  toValue,
  type InjectionKey,
  type MaybeRefOrGetter,
} from "vue";

export type MinimalRoute = {
  params: Record<string, unknown>;
  name?: unknown;
};

/**
 * A database to run queries on instead of the one in the route; `admin` runs
 * them through the admin endpoints.
 */
export type DbTarget = { owner: string; db: string; admin?: boolean };

const DB_TARGET_KEY: InjectionKey<MaybeRefOrGetter<DbTarget>> =
  Symbol("dbTarget");

/**
 * Makes the query components below run on `target` instead of the route's
 * database (e.g. a dashboard widget querying its own database).
 */
export const provideDbTarget = (target: MaybeRefOrGetter<DbTarget>): void =>
  provide(DB_TARGET_KEY, target);

/**
 * Resolves the current route, or the provided database target. Falls back to
 * an empty route outside a router context.
 */
export const useMinimalRoute = (): MinimalRoute => {
  const target = hasInjectionContext()
    ? inject(DB_TARGET_KEY, undefined)
    : undefined;
  if (target !== undefined) {
    return {
      // Read on use, so a target that changes is followed.
      get params() {
        const { owner, db } = toValue(target);
        return { owner, db };
      },
      get name() {
        return toValue(target).admin ? "admin-query" : "query";
      },
    };
  }
  try {
    // `useRoute` returns undefined (with a warning) outside of a router.
    const route: MinimalRoute | undefined = useRoute();
    return route ?? { params: {}, name: undefined };
  } catch {
    return { params: {}, name: undefined };
  }
};

export const getDbParams = (
  route: MinimalRoute,
): { owner: string; db: string } | undefined => {
  const ownerParam = route.params["owner"];
  const dbParam = route.params["db"];
  const owner = typeof ownerParam === "string" ? ownerParam : "";
  const db = typeof dbParam === "string" ? dbParam : "";
  if (!owner || !db) {
    return undefined;
  }
  return { owner, db };
};

export const isAdminRoute = (route: MinimalRoute): boolean =>
  String(route.name ?? "").startsWith("admin-");

/**
 * Executes a batch of queries against the database identified by the route
 * (`owner`/`db` params). Uses the admin endpoints on `admin-*` routes.
 * Returns `undefined` when the route does not identify a database.
 */
export const executeDbQueries = async (
  queries: QueryType[],
  mutation: boolean,
  route: MinimalRoute,
  { quiet = false }: { quiet?: boolean } = {},
): Promise<QueryResult[] | undefined> => {
  checkClient(client);

  const params = getDbParams(route);
  if (!params) {
    return undefined;
  }

  // Quiet runs show their own errors, without the global notification.
  const config = quiet ? { quietErrors: true } : undefined;
  let response: { data: QueryResult[] };

  if (isAdminRoute(route)) {
    response = mutation
      ? await client.value.admin_db_exec_mut(params, queries, config)
      : await client.value.admin_db_exec(params, queries, config);
  } else {
    response = mutation
      ? await client.value.db_exec_mut(params, queries, config)
      : await client.value.db_exec(params, queries, config);
  }

  return response.data;
};

/** Composable binding `executeDbQueries` to the current route (or provided target). */
export const useDbExec = () => {
  const route = useMinimalRoute();
  return {
    route,
    exec: (queries: QueryType[]) => executeDbQueries(queries, false, route),
    execMut: (queries: QueryType[]) => executeDbQueries(queries, true, route),
  };
};
