import {
  Comparison,
  CountComparison,
  DbKeyOrder,
  QueryBuilder as ApiQueryBuilder,
  type Components,
} from "@agnesoft/agdb_api";
import { getQueryStepLabel } from "../mock/queryApiMock";
import type {
  QueryStep as QueryStepModel,
  QueryStepArgEntry,
  QueryStepFieldValue,
} from "../composables/types";

export type BuilderDbValueInput =
  | string
  | boolean
  | number
  | number[]
  | string[]
  | Components.Schemas.DbValue;

const parseBoolean = (value: string | undefined): boolean => {
  return String(value ?? "").toLowerCase() === "true";
};

const parseArray = (value: string | undefined): string[] => {
  return (value ?? "")
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
};

const parseNumberArray = (value: string | undefined): number[] => {
  return parseArray(value).map((item) => Number(item));
};

const parseBooleanArray = (value: string | undefined): boolean[] => {
  return parseArray(value).map((item) => parseBoolean(item));
};

const parseDbValue = (
  field: QueryStepFieldValue | undefined,
): BuilderDbValueInput => {
  if (!field) return "";

  switch (field.selectedOption) {
    case "string":
      return field.value ?? "";
    case "signed":
      return Number(field.value ?? 0);
    case "unsigned":
      return { U64: Number(field.value ?? 0) };
    case "float":
      return { F64: Number(field.value ?? 0) };
    case "boolean":
      return parseBoolean(field.value);
    case "string[]":
      return parseArray(field.value);
    case "signed[]":
      return parseNumberArray(field.value);
    case "unsigned[]":
      return { VecU64: parseNumberArray(field.value) };
    case "float[]":
      return { VecF64: parseNumberArray(field.value) };
    case "boolean[]":
      return parseBooleanArray(field.value).map(String);
    default:
      return field.value ?? field.selectedOption;
  }
};

const parseIdValue = (
  field: QueryStepFieldValue | undefined,
): number | string => {
  if (!field) return 0;
  if (field.selectedOption === "signed") {
    return Number(field.value ?? 0);
  }

  return field.value ?? "";
};

const parseNumberValue = (field: QueryStepFieldValue | undefined): number => {
  return Number(field?.value ?? 0);
};

const parseEntriesToIds = (
  entries: QueryStepArgEntry[],
): (number | string)[] => {
  return entries.map((entry) => parseIdValue(entry[0]));
};

const parseEntriesToValues = (
  entries: QueryStepArgEntry[],
): BuilderDbValueInput[] => {
  return entries.map((entry) => parseDbValue(entry[0]));
};

const toCountComparison = (
  kind: string,
  value: number,
): Components.Schemas.CountComparison => {
  switch (kind) {
    case "greaterThan":
      return CountComparison.GreaterThan(value);
    case "greaterThanOrEqual":
      return CountComparison.GreaterThanOrEqual(value);
    case "lessThan":
      return CountComparison.LessThan(value);
    case "lessThanOrEqual":
      return CountComparison.LessThanOrEqual(value);
    case "notEqual":
      return CountComparison.NotEqual(value);
    case "equal":
    default:
      return CountComparison.Equal(value);
  }
};

const toComparison = (
  kind: string,
  value: BuilderDbValueInput,
): Components.Schemas.Comparison => {
  switch (kind) {
    case "greaterThan":
      return Comparison.GreaterThan(value);
    case "greaterThanOrEqual":
      return Comparison.GreaterThanOrEqual(value);
    case "lessThan":
      return Comparison.LessThan(value);
    case "lessThanOrEqual":
      return Comparison.LessThanOrEqual(value);
    case "notEqual":
      return Comparison.NotEqual(value);
    case "contains":
      return Comparison.Contains(value);
    case "startsWith":
      return Comparison.StartsWith(value);
    case "endsWith":
      return Comparison.EndsWith(value);
    case "equal":
    default:
      return Comparison.Equal(value);
  }
};

const getStepArgs = (step: QueryStepModel): unknown[] => {
  const entries = step.args ?? [];
  const type = step.type;

  if (type.endsWith(".aliases")) {
    return [entries.map((entry) => String(entry[0]?.value ?? ""))];
  }

  if (type.endsWith(".ids")) {
    const ids = parseEntriesToIds(entries);
    return [ids.length === 1 ? ids[0] : ids];
  }

  if (
    type.endsWith(".count") ||
    type.endsWith(".limit") ||
    type.endsWith(".offset")
  ) {
    return [parseNumberValue(entries[0]?.[0])];
  }

  if (type.endsWith(".values_uniform")) {
    const pairs = entries.map((entry) => [
      parseDbValue(entry[0]),
      parseDbValue(entry[1]),
    ]);
    return [pairs];
  }

  if (type.endsWith(".values")) {
    if (type === "select.values" || type === "remove.values") {
      return [parseEntriesToValues(entries)];
    }

    const pairs = entries.map((entry) => [
      parseDbValue(entry[0]),
      parseDbValue(entry[1]),
    ]);

    return [pairs.map((pair) => [pair])];
  }

  if (type.endsWith(".from") || type.endsWith(".to")) {
    return [parseIdValue(entries[0]?.[0])];
  }

  if (type === "search.order_by") {
    const orders = entries.map((entry) => {
      const direction = entry[0]?.selectedOption;
      const value = parseDbValue(entry[1]);
      return direction === "desc"
        ? DbKeyOrder.Desc(value)
        : DbKeyOrder.Asc(value);
    });
    return [orders.length === 1 ? orders[0] : orders];
  }

  if (
    type === "search.where.distance" ||
    type === "search.where.edge_count" ||
    type === "search.where.edge_count_from" ||
    type === "search.where.edge_count_to"
  ) {
    const comparisonType = entries[0]?.[0]?.selectedOption ?? "equal";
    const value = parseNumberValue(entries[0]?.[1]);
    return [toCountComparison(comparisonType, value)];
  }

  if (type === "search.where.key.value") {
    const comparisonType = entries[0]?.[0]?.selectedOption ?? "equal";
    const value = parseDbValue(entries[0]?.[1]);
    return [toComparison(comparisonType, value)];
  }

  if (type.endsWith(".keys")) {
    return [parseEntriesToValues(entries)];
  }

  if (
    type.endsWith(".index") ||
    type.endsWith(".value") ||
    type.endsWith(".key")
  ) {
    return [parseDbValue(entries[0]?.[0])];
  }

  return [];
};

export const buildQueryFromSteps = (
  querySteps: QueryStepModel[],
): Components.Schemas.QueryType => {
  if (!querySteps.length) {
    throw new Error("No query steps to execute");
  }

  let chain: unknown = ApiQueryBuilder;
  let terminated = false;

  for (const step of querySteps) {
    if (step.invalid) {
      throw new Error("Cannot execute query with invalid steps");
    }

    const methodName = getQueryStepLabel(step.type);
    if (methodName === "query") {
      const queryFn = (chain as { query?: () => unknown }).query;
      if (typeof queryFn !== "function") {
        throw new Error(
          "Current query chain cannot be terminated with query()",
        );
      }
      chain = queryFn.call(chain);
      terminated = true;
      continue;
    }

    const method = (chain as Record<string, (...args: unknown[]) => unknown>)[
      methodName
    ];
    if (typeof method !== "function") {
      throw new Error(
        `Unsupported step '${methodName}' for current query chain`,
      );
    }

    chain = method.apply(chain, getStepArgs(step));
  }

  if (!terminated) {
    const queryFn = (chain as { query?: () => unknown }).query;
    if (typeof queryFn !== "function") {
      throw new Error("Query chain is not terminable. Add a terminal step.");
    }
    chain = queryFn.call(chain);
  }

  return chain as Components.Schemas.QueryType;
};
