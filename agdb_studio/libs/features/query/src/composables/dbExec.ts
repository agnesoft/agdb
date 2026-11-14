import type { Components } from "@agnesoft/agdb_api";
import { checkClient, client } from "@agdb-studio/api/src/api";
import { useRoute } from "vue-router";

export type MinimalRoute = {
  params: Record<string, unknown>;
  name?: unknown;
};

/** Resolves the current route, falling back to an empty one outside a router context. */
export const useMinimalRoute = (): MinimalRoute => {
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
  queries: Components.Schemas.QueryType[],
  mutation: boolean,
  route: MinimalRoute,
): Promise<Components.Schemas.QueryResult[] | undefined> => {
  checkClient(client);

  const params = getDbParams(route);
  if (!params) {
    return undefined;
  }

  let response: { data: Components.Schemas.QueryResult[] };

  if (isAdminRoute(route)) {
    response = mutation
      ? await client.value.admin_db_exec_mut(params, queries)
      : await client.value.admin_db_exec(params, queries);
  } else {
    response = mutation
      ? await client.value.db_exec_mut(params, queries)
      : await client.value.db_exec(params, queries);
  }

  return response.data;
};

/** Composable binding `executeDbQueries` to the current route. */
export const useDbExec = () => {
  const route = useMinimalRoute();
  return {
    route,
    exec: (queries: Components.Schemas.QueryType[]) =>
      executeDbQueries(queries, false, route),
    execMut: (queries: Components.Schemas.QueryType[]) =>
      executeDbQueries(queries, true, route),
  };
};
