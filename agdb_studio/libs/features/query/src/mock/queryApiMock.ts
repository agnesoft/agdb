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

/** Convenience method suffixes for count conditions (e.g. `distance_greater_than`). Equality is the base method. */
export const COUNT_COMPARISONS = [
  "greater_than",
  "greater_than_or_equal",
  "less_than",
  "less_than_or_equal",
  "not_equal",
] as const;

/** Convenience methods following `key(KEY)`. Equality is `value()`. */
export const COMPARISONS = [
  "value",
  ...COUNT_COMPARISONS,
  "contains",
  "starts_with",
  "ends_with",
  "any",
  "regex",
] as const;

/** Conditions that take a count and have count comparison convenience methods. */
const COUNT_CONDITIONS = [
  "distance",
  "edge_count",
  "edge_count_from",
  "edge_count_to",
] as const;

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
  label: string;
  followers: string[];
  arguments?: QueryArguments;
};

const node = (
  label: string,
  followers: string[],
  arguments_: QueryArguments | undefined = undefined,
): QueryApiMockType => ({
  label,
  followers,
  arguments: arguments_,
});

const ID_ARGUMENTS: QueryArguments = {
  fields: [{ options: IDS }],
  repeatable: true,
};

const VALUE_ARGUMENTS: QueryArguments = {
  fields: [{ options: VALUE_TYPES }],
  repeatable: false,
};

const KV_ARGUMENTS: QueryArguments = {
  fields: [{ options: VALUE_TYPES }, { options: VALUE_TYPES }],
  repeatable: true,
};

const COUNT_ARGUMENTS: QueryArguments = {
  fields: [{ options: NUMBER }],
  repeatable: false,
};

const ORDER_BY_ARGUMENTS: QueryArguments = {
  fields: [{ options: ORDER_DIRECTIONS }, { options: VALUE_TYPES }],
  repeatable: true,
};

/** Regex patterns are only matched when given as a string. */
const STRING_VALUE_ARGUMENTS: QueryArguments = {
  fields: [{ options: ["string"] }],
  repeatable: false,
};

const LIST_VALUE_ARGUMENTS: QueryArguments = {
  fields: [{ options: VALUE_TYPES }],
  repeatable: true,
};

const STRING_LIST_ARGUMENTS: QueryArguments = {
  fields: [{ options: ["string"] }],
  repeatable: true,
};

const WHERE_BUILDERS = [
  "search.where",
  "search.where.not",
  "search.where.beyond",
  "search.where.not_beyond",
  "search.where.where",
  "search.where.and",
  "search.where.or",
] as const;

const WHERE_LOGIC = [
  "search.where.and",
  "search.where.or",
  "search.where.end_where",
  "query",
] as const;

type CountConditionMethod =
  | (typeof COUNT_CONDITIONS)[number]
  | `${(typeof COUNT_CONDITIONS)[number]}_${(typeof COUNT_COMPARISONS)[number]}`;

const COUNT_CONDITION_METHODS: readonly CountConditionMethod[] =
  COUNT_CONDITIONS.flatMap((condition) => [
    condition,
    ...COUNT_COMPARISONS.map(
      (comparison) => `${condition}_${comparison}` as const,
    ),
  ]);

/** Step types taking a single count, e.g. `search.where.distance_greater_than`. */
export const WHERE_COUNT_CONDITION_STEPS = COUNT_CONDITION_METHODS.map(
  (method) => `search.where.${method}` as const,
);

/** Step types following `key(KEY)` taking a single value, e.g. `search.where.key.contains`. */
export const WHERE_KEY_COMPARISON_STEPS = COMPARISONS.map(
  (comparison) => `search.where.key.${comparison}` as const,
);

const WHERE_CONDITIONS = [
  ...WHERE_COUNT_CONDITION_STEPS,
  "search.where.edge",
  "search.where.ids",
  "search.where.key",
  "search.where.keys",
  "search.where.neighbor",
  "search.where.node",
] as const;

export const queryApiMock: Record<string, QueryApiMockType> = {
  exec: node("exec", ["select", "search"]),
  exec_mut: node("exec_mut", ["insert", "remove"]),
  query: node("query", []),

  insert: node("insert", [
    "insert.aliases",
    "insert.edges",
    "insert.index",
    "insert.nodes",
    "insert.values",
    "insert.values_uniform",
    "insert.amend",
    "insert.amend_uniform",
    "insert.amend_or",
    "insert.amend_or_uniform",
    "insert.amend_and",
    "insert.amend_and_uniform",
    "insert.amend_xor",
    "insert.amend_xor_uniform",
  ]),
  "insert.aliases": node(
    "aliases",
    ["insert.aliases.ids"],
    STRING_LIST_ARGUMENTS,
  ),
  "insert.aliases.ids": node("ids", ["query"], ID_ARGUMENTS),
  "insert.edges": node("edges", ["insert.edges.from", "insert.edges.ids"]),
  "insert.edges.from": node("from", ["insert.edges.from.to"], ID_ARGUMENTS),
  "insert.edges.ids": node("ids", ["insert.edges.from"], ID_ARGUMENTS),
  "insert.edges.from.to": node(
    "to",
    [
      "insert.edges.from.to.each",
      "insert.edges.from.to.values",
      "insert.edges.from.to.values_uniform",
      "query",
    ],
    ID_ARGUMENTS,
  ),
  "insert.edges.from.to.each": node("each", [
    "insert.edges.from.to.values",
    "insert.edges.from.to.values_uniform",
    "query",
  ]),
  "insert.edges.from.to.values": node("values", ["query"], KV_ARGUMENTS),
  "insert.edges.from.to.values_uniform": node(
    "values_uniform",
    ["query"],
    KV_ARGUMENTS,
  ),
  "insert.index": node("index", ["query"], VALUE_ARGUMENTS),
  "insert.nodes": node("nodes", [
    "insert.nodes.aliases",
    "insert.nodes.count",
    "insert.nodes.ids",
    "insert.nodes.values",
    "insert.nodes.values_uniform",
  ]),
  "insert.nodes.aliases": node(
    "aliases",
    ["insert.nodes.values", "insert.nodes.values_uniform", "query"],
    STRING_LIST_ARGUMENTS,
  ),
  "insert.nodes.count": node(
    "count",
    ["insert.nodes.values_uniform", "query"],
    COUNT_ARGUMENTS,
  ),
  "insert.nodes.ids": node(
    "ids",
    [
      "insert.nodes.aliases",
      "insert.nodes.count",
      "insert.nodes.values",
      "insert.nodes.values_uniform",
    ],
    ID_ARGUMENTS,
  ),
  "insert.nodes.values": node("values", ["query"], KV_ARGUMENTS),
  "insert.nodes.values_uniform": node(
    "values_uniform",
    ["query"],
    KV_ARGUMENTS,
  ),
  "insert.values": node(
    "values",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.values_uniform": node(
    "values_uniform",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.values.ids": node("ids", ["query"], ID_ARGUMENTS),
  "insert.amend": node("amend", ["insert.values.ids", "search"], KV_ARGUMENTS),
  "insert.amend_uniform": node(
    "amend_uniform",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.amend_or": node(
    "amend_or",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.amend_or_uniform": node(
    "amend_or_uniform",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.amend_and": node(
    "amend_and",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.amend_and_uniform": node(
    "amend_and_uniform",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.amend_xor": node(
    "amend_xor",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),
  "insert.amend_xor_uniform": node(
    "amend_xor_uniform",
    ["insert.values.ids", "search"],
    KV_ARGUMENTS,
  ),

  remove: node("remove", [
    "remove.aliases",
    "remove.ids",
    "search",
    "remove.index",
    "remove.values",
    "remove.amend",
    "remove.amend_uniform",
    "remove.amend_or",
    "remove.amend_or_uniform",
    "remove.amend_and",
    "remove.amend_and_uniform",
    "remove.amend_xor",
    "remove.amend_xor_uniform",
  ]),
  "remove.aliases": node("aliases", ["query"], STRING_LIST_ARGUMENTS),
  "remove.ids": node("ids", ["query"], ID_ARGUMENTS),
  "remove.index": node("index", ["query"], VALUE_ARGUMENTS),
  "remove.values": node(
    "values",
    ["remove.values.ids", "search"],
    LIST_VALUE_ARGUMENTS,
  ),
  "remove.values.ids": node("ids", ["query"], ID_ARGUMENTS),
  "remove.amend": node("amend", ["remove.amend.ids", "search"], KV_ARGUMENTS),
  "remove.amend_uniform": node(
    "amend_uniform",
    ["remove.amend.ids", "search"],
    KV_ARGUMENTS,
  ),
  "remove.amend_or": node(
    "amend_or",
    ["remove.amend.ids", "search"],
    KV_ARGUMENTS,
  ),
  "remove.amend_or_uniform": node(
    "amend_or_uniform",
    ["remove.amend.ids", "search"],
    KV_ARGUMENTS,
  ),
  "remove.amend_and": node(
    "amend_and",
    ["remove.amend.ids", "search"],
    KV_ARGUMENTS,
  ),
  "remove.amend_and_uniform": node(
    "amend_and_uniform",
    ["remove.amend.ids", "search"],
    KV_ARGUMENTS,
  ),
  "remove.amend_xor": node(
    "amend_xor",
    ["remove.amend.ids", "search"],
    KV_ARGUMENTS,
  ),
  "remove.amend_xor_uniform": node(
    "amend_xor_uniform",
    ["remove.amend.ids", "search"],
    KV_ARGUMENTS,
  ),
  "remove.amend.ids": node("ids", ["query"], ID_ARGUMENTS),

  select: node("select", [
    "select.aliases",
    "select.edge_count",
    "select.edge_count_from",
    "select.edge_count_to",
    "select.ids",
    "select.indexes",
    "select.keys",
    "select.key_count",
    "select.node_count",
    "search",
    "select.values",
  ]),
  "select.aliases": node("aliases", ["select.aliases.ids", "search", "query"]),
  "select.aliases.ids": node("ids", ["query"], ID_ARGUMENTS),
  "select.edge_count": node("edge_count", [
    "select.edge_count.ids",
    "search",
    "query",
  ]),
  "select.edge_count.ids": node("ids", ["query"], ID_ARGUMENTS),
  "select.edge_count_from": node("edge_count_from", [
    "select.edge_count_from.ids",
    "search",
    "query",
  ]),
  "select.edge_count_from.ids": node("ids", ["query"], ID_ARGUMENTS),
  "select.edge_count_to": node("edge_count_to", [
    "select.edge_count_to.ids",
    "search",
    "query",
  ]),
  "select.edge_count_to.ids": node("ids", ["query"], ID_ARGUMENTS),
  "select.ids": node("ids", ["query"], ID_ARGUMENTS),
  "select.indexes": node("indexes", ["query"]),
  "select.keys": node("keys", ["select.keys.ids", "search", "query"]),
  "select.keys.ids": node("ids", ["query"], ID_ARGUMENTS),
  "select.key_count": node("key_count", [
    "select.key_count.ids",
    "search",
    "query",
  ]),
  "select.key_count.ids": node("ids", ["query"], ID_ARGUMENTS),
  "select.node_count": node("node_count", ["query"]),
  "select.values": node(
    "values",
    ["select.values.ids", "search", "query"],
    LIST_VALUE_ARGUMENTS,
  ),
  "select.values.ids": node("ids", ["query"], ID_ARGUMENTS),

  search: node("search", [
    "search.breadth_first",
    "search.depth_first",
    "search.elements",
    "search.from",
    "search.index",
    "search.to",
  ]),
  "search.breadth_first": node("breadth_first", ["search.from", "search.to"]),
  "search.depth_first": node("depth_first", ["search.from", "search.to"]),
  "search.elements": node("elements", [
    "search.limit",
    "search.offset",
    "search.order_by",
    "search.where",
    "query",
  ]),
  "search.from": node(
    "from",
    [
      "search.to",
      "search.limit",
      "search.offset",
      "search.order_by",
      "search.where",
      "query",
    ],
    {
      fields: [{ options: IDS }],
      repeatable: false,
    },
  ),
  "search.to": node(
    "to",
    [
      "search.limit",
      "search.offset",
      "search.order_by",
      "search.where",
      "query",
    ],
    {
      fields: [{ options: IDS }],
      repeatable: false,
    },
  ),
  "search.index": node("index", ["search.index.value"], VALUE_ARGUMENTS),
  "search.index.value": node("value", ["query"], VALUE_ARGUMENTS),
  "search.order_by": node(
    "order_by",
    ["search.limit", "search.offset", "search.where", "query"],
    ORDER_BY_ARGUMENTS,
  ),
  "search.offset": node(
    "offset",
    ["search.limit", "search.where", "query"],
    COUNT_ARGUMENTS,
  ),
  "search.limit": node("limit", ["search.where", "query"], COUNT_ARGUMENTS),

  "search.where": node("where", [...WHERE_BUILDERS, ...WHERE_CONDITIONS]),
  "search.where.not": node("not", [...WHERE_BUILDERS, ...WHERE_CONDITIONS]),
  "search.where.beyond": node("beyond", [
    ...WHERE_BUILDERS,
    ...WHERE_CONDITIONS,
  ]),
  "search.where.not_beyond": node("not_beyond", [
    ...WHERE_BUILDERS,
    ...WHERE_CONDITIONS,
  ]),
  "search.where.where": node("where", [...WHERE_BUILDERS, ...WHERE_CONDITIONS]),
  "search.where.and": node("and", [...WHERE_BUILDERS, ...WHERE_CONDITIONS]),
  "search.where.or": node("or", [...WHERE_BUILDERS, ...WHERE_CONDITIONS]),
  "search.where.end_where": node("end_where", [...WHERE_LOGIC]),
  ...Object.fromEntries(
    COUNT_CONDITION_METHODS.map((method, i) => [
      WHERE_COUNT_CONDITION_STEPS[i],
      node(method, [...WHERE_LOGIC], COUNT_ARGUMENTS),
    ]),
  ),
  "search.where.edge": node("edge", [...WHERE_LOGIC]),
  "search.where.ids": node("ids", [...WHERE_LOGIC], ID_ARGUMENTS),
  "search.where.key": node(
    "key",
    [...WHERE_KEY_COMPARISON_STEPS],
    VALUE_ARGUMENTS,
  ),
  ...Object.fromEntries(
    COMPARISONS.map((comparison, i) => [
      WHERE_KEY_COMPARISON_STEPS[i],
      node(
        comparison,
        [...WHERE_LOGIC],
        comparison === "regex" ? STRING_VALUE_ARGUMENTS : VALUE_ARGUMENTS,
      ),
    ]),
  ),
  "search.where.keys": node("keys", [...WHERE_LOGIC], LIST_VALUE_ARGUMENTS),
  "search.where.neighbor": node("neighbor", [...WHERE_LOGIC]),
  "search.where.node": node("node", [...WHERE_LOGIC]),
};

export const getQueryStepLabel = (type: string): string => {
  return queryApiMock[type]?.label ?? type;
};
