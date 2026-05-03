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

/** The kind of value an option expects from the user, or null if it takes no value. */
export type OptionValueType =
  | "string"
  | "number"
  | "boolean"
  | "string[]"
  | "number[]"
  | "boolean[]"
  | null;

/** Maps every possible option name to the value type the user must enter, or null for no value. */
export const OPTION_TYPE_MAP: Record<string, OptionValueType> = {
  // VALUE_TYPES
  string: "string",
  unsigned: "number",
  signed: "number",
  boolean: "boolean",
  float: "number",
  "string[]": "string[]",
  "unsigned[]": "number[]",
  "signed[]": "number[]",
  "boolean[]": "boolean[]",
  "float[]": "number[]",
  // ORDER_DIRECTIONS
  asc: null,
  desc: null,
  // COUNT_COMPARISONS / COMPARISON
  equal: null,
  greaterThan: null,
  greaterThanOrEqual: null,
  lessThan: null,
  lessThanOrEqual: null,
  notEqual: null,
  contains: null,
  startsWith: null,
  endsWith: null,
};

/** Maps option names to compact labels used in query argument dropdowns. */
export const OPTION_SHORTCUT_MAP: Record<string, string> = {
  // VALUE_TYPES / IDS / NUMBER
  string: "s",
  unsigned: "u",
  signed: "i",
  boolean: "b",
  float: "f",
  "string[]": "s[]",
  "unsigned[]": "u[]",
  "signed[]": "i[]",
  "boolean[]": "b[]",
  "float[]": "f[]",
  // ORDER_DIRECTIONS
  asc: "↑",
  desc: "↓",
  // COUNT_COMPARISONS / COMPARISON
  equal: "=",
  greaterThan: ">",
  greaterThanOrEqual: "≥",
  lessThan: "<",
  lessThanOrEqual: "≤",
  notEqual: "≠",
  contains: "∋",
  startsWith: "⊢",
  endsWith: "⊣",
};

export type ArgumentField = {
  /** The set of options the user can choose from (type selector). */
  options: readonly string[];
};

export type QueryArguments = {
  fields: ArgumentField[];
  repeatable: boolean;
};

export type QueryApiMockType = {
  followers: string[];
  arguments?: QueryArguments;
};

export const queryApiMock: Record<string, QueryApiMockType> = {
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
      fields: [{ options: VALUE_TYPES }, { options: VALUE_TYPES }],
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
      fields: [{ options: IDS }],
      repeatable: false,
    },
  },
  orderBy: {
    followers: ["limit", "values"],
    arguments: {
      fields: [{ options: ORDER_DIRECTIONS }, { options: VALUE_TYPES }],
      repeatable: false,
    },
  },
  distance: {
    followers: ["limit", "values"],
    arguments: {
      fields: [{ options: COUNT_COMPARISONS }, { options: NUMBER }],
      repeatable: false,
    },
  },
  compare: {
    followers: ["limit", "values"],
    arguments: {
      fields: [{ options: COMPARISON }, { options: VALUE_TYPES }],
      repeatable: false,
    },
  },
  limit: {
    followers: [],
    arguments: {
      fields: [{ options: NUMBER }],
      repeatable: false,
    },
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
