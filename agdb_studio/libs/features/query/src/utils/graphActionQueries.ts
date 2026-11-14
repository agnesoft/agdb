import {
  CountComparison,
  QueryBuilder,
  type Components,
} from "@agnesoft/agdb_api";

type DbKeyValue = Components.Schemas.DbKeyValue;
type QueryType = Components.Schemas.QueryType;

/** Selects elements with all their stored values. */
export const selectElementQuery = (ids: number | number[]): QueryType =>
  QueryBuilder.select().ids(ids).query();

/**
 * Like `selectElementQuery`, but returns only the elements that exist rather
 * than failing on ids deleted meanwhile. It scans all elements, so it is only
 * a fallback.
 */
export const selectExistingElementsQuery = (ids: number[]): QueryType =>
  QueryBuilder.select().search().elements().where().ids(ids).query();

/** Removes an element. Removing a node also removes all its edges. */
export const deleteElementQuery = (id: number): QueryType =>
  QueryBuilder.remove().ids(id).query();

const keyId = (key: Components.Schemas.DbValue): string => JSON.stringify(key);

/**
 * Builds the mutations that turn `original` values into `edited` values:
 * removes keys that no longer exist and inserts (upserts) new or changed values.
 */
export const updateValuesQueries = (
  id: number,
  original: DbKeyValue[],
  edited: DbKeyValue[],
): QueryType[] => {
  const editedKeys = new Set(edited.map((kv) => keyId(kv.key)));
  const removedKeys = original
    .map((kv) => kv.key)
    .filter((key) => !editedKeys.has(keyId(key)));

  // Only write values that are new or changed.
  const originalValues = new Set(original.map((kv) => JSON.stringify(kv)));
  const changed = edited.filter(
    (kv) => !originalValues.has(JSON.stringify(kv)),
  );

  const queries: QueryType[] = [];
  if (removedKeys.length > 0) {
    queries.push(QueryBuilder.remove().values(removedKeys).ids(id).query());
  }
  if (changed.length > 0) {
    queries.push(QueryBuilder.insert().values([changed]).ids(id).query());
  }
  return queries;
};

/** Inserts a single edge `from` → `to`, optionally with values. */
export const insertEdgeQuery = (
  from: number,
  to: number,
  values: DbKeyValue[] = [],
): QueryType => {
  const builder = QueryBuilder.insert().edges().from(from).to(to);
  return values.length > 0
    ? builder.values_uniform(values).query()
    : builder.query();
};

/**
 * Selects direct neighbours of a node in both directions:
 * outgoing and incoming edges plus the nodes on their other end.
 */
export const neighboursQueries = (id: number): QueryType[] => [
  QueryBuilder.select()
    .search()
    .from(id)
    .where()
    .distance(CountComparison.LessThanOrEqual(2))
    .query(),
  QueryBuilder.select()
    .search()
    .to(id)
    .where()
    .distance(CountComparison.LessThanOrEqual(2))
    .query(),
];
