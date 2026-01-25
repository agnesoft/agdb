import { a } from "node_modules/vitest/dist/chunks/suite.d.BJWk38HB";

export const VALUE_TYPES = [
  "string",
  "unsigned",
  "signed",
  "boolean",
  "float",
  "string[]",
  "unsigned[]",
  "signed[]",
  "boolean[]",
  "float[]",
] as const;

export type ValueType = (typeof VALUE_TYPES)[number];

export const ORDER_DIRECTIONS = ["asc", "desc"] as const;

export type OrderDirection = (typeof ORDER_DIRECTIONS)[number];

export const COUNT_COMPARISONS = [
  "equal",
  "greaterThan",
  "greaterThanOrEqual",
  "lessThan",
  "lessThanOrEqual",
  "notEqual",
] as const;

export type CountComparison = (typeof COUNT_COMPARISONS)[number];

export const COMPARISON = [
  ...COUNT_COMPARISONS,
  "contains",
  "startsWith",
  "endsWith",
] as const;

export type Comparison = (typeof COMPARISON)[number];

export const IDS = ["signed", "string"] as const;

export type IdType = (typeof IDS)[number];

export const NUMBER = ["unsigned"] as const;

export type NumberType = (typeof NUMBER)[number];

export const queryApiMock = {
  exec: {
    followers: ["select", "search"],
  },
  exec_mut: {
    followers: ["insert", "update", "delete"],
  },
  select: {
    followers: ["key_count", "search", "from", "limit", "values"],
  },
  values: {
    followers: ["key_count", "search", "from", "limit", "values"],
    arguments: {
      schema: [VALUE_TYPES, VALUE_TYPES],
      repeatable: true,
    },
  },
  key_count: {
    followers: ["search", "from", "limit", "values"],
  },
  search: {
    followers: ["from", "limit", "values"],
  },
  from: {
    followers: ["limit", "values", "orderBy", "distance", "compare"],
    arguments: {
      schema: [IDS],
      repeatable: false,
    },
  },
  orderBy: {
    followers: ["limit", "values"],
    arguments: {
      schema: [ORDER_DIRECTIONS, VALUE_TYPES],
      repeatable: false,
    },
  },
  distance: {
    followers: ["limit", "values"],
    arguments: {
      schema: [COUNT_COMPARISONS, NUMBER],
      repeatable: false,
    },
  },
  compare: {
    followers: ["limit", "values"],
    arguments: {
      schema: [COMPARISON, VALUE_TYPES],
      repeatable: false,
    },
  },
  limit: {
    followers: [],
    values: ["number"],
  },
  insert: {
    followers: ["values"],
  },
  update: {
    followers: ["search", "values"],
  },
  delete: {
    followers: ["search"],
  },
};
