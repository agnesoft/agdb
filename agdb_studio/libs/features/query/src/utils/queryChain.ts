import {
  IDS,
  NUMBER,
  ORDER_DIRECTIONS,
  getQueryStepLabel,
  queryApiMock,
} from "../mock/queryApiMock";
import type {
  CopiedStep,
  QueryStep,
  QueryStepArgEntry,
  QueryStepFieldValue,
  QueryType,
} from "../composables/types";

/**
 * A language-neutral literal in a query chain. Each code language
 * (e.g. Rust) formats and parses these in its own syntax.
 */
export type Literal =
  | { kind: "string"; value: string }
  /** `int` is an integer without an explicit type, e.g. an id or a count. */
  | {
      kind: "number";
      value: string;
      type: "int" | "signed" | "unsigned" | "float";
    }
  | { kind: "boolean"; value: boolean }
  /**
   * Several arguments of one parameter, e.g. ids or key-values. `of` tells
   * languages that need one item type what to convert mixed items to.
   */
  | { kind: "list"; items: Literal[]; of?: "ids" | "values" }
  /** A single array value, e.g. `VecI64`. */
  | { kind: "array"; items: Literal[] }
  | { kind: "pair"; key: Literal; value: Literal }
  | { kind: "order"; direction: "asc" | "desc"; key: Literal }
  /** A comparison object, e.g. `LessThan(3)`; `op` is snake_case. */
  | { kind: "comparison"; op: string; value: Literal };

/** One method call of a query chain; `method` is the builder's step label. */
export type ChainCall = { method: string; args: Literal[] };

export type QueryLanguage = {
  name: string;
  format: (calls: ChainCall[]) => string;
  parse: (text: string) => { calls: ChainCall[] } | { error: string };
};

type ArgShape =
  | "none"
  | "ids"
  | "id"
  | "count"
  | "value"
  | "values"
  | "strings"
  | "kv"
  | "orders";

const argShape = (type: QueryType): ArgShape => {
  const schema = queryApiMock[type]?.arguments;
  const [first, second] = schema?.fields ?? [];
  if (!schema || !first) return "none";
  if (second) return first.options === ORDER_DIRECTIONS ? "orders" : "kv";
  if (first.options === IDS) return schema.repeatable ? "ids" : "id";
  if (first.options === NUMBER) return "count";
  if (schema.repeatable) {
    return first.options.length === 1 ? "strings" : "values";
  }
  return "value";
};

const allowedOptions = (type: QueryType, field = 0): readonly string[] =>
  /* v8 ignore next -- @preserve: only called for steps with arguments */
  queryApiMock[type]?.arguments?.fields[field]?.options ?? [];

// --- steps -> chain -------------------------------------------------------

const scalar = (option: string, value: string): Literal => {
  if (option === "string") return { kind: "string", value };
  if (option === "boolean") {
    return { kind: "boolean", value: value.trim().toLowerCase() === "true" };
  }
  const type = option === "float" || option === "unsigned" ? option : "signed";
  return { kind: "number", value: value.trim() || "0", type };
};

const valueLiteral = (field: QueryStepFieldValue | undefined): Literal => {
  const option = field?.selectedOption ?? "string";
  const value = field?.value ?? "";
  if (!option.endsWith("[]")) return scalar(option, value);
  const items = value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length);
  return {
    kind: "array",
    items: items.map((item) => scalar(option.slice(0, -2), item)),
  };
};

const idLiteral = (field: QueryStepFieldValue | undefined): Literal =>
  field?.selectedOption === "string"
    ? { kind: "string", value: field.value ?? "" }
    : { kind: "number", value: field?.value?.trim() || "0", type: "int" };

const oneOrList = (items: Literal[], of?: "ids"): Literal =>
  items.length === 1 && items[0] ? items[0] : { kind: "list", items, of };

const pairOf = (entry: QueryStepArgEntry): Literal => ({
  kind: "pair",
  key: valueLiteral(entry[0]),
  value: valueLiteral(entry[1]),
});

const stepArgs = (step: CopiedStep): Literal[] => {
  const entries = step.args ?? [];
  if (!entries.length) return [];
  switch (argShape(step.type)) {
    case "ids":
      return [
        oneOrList(
          entries.map((entry) => idLiteral(entry[0])),
          "ids",
        ),
      ];
    case "id":
      return [idLiteral(entries[0]?.[0])];
    case "count":
      return [
        {
          kind: "number",
          value: entries[0]?.[0]?.value?.trim() || "0",
          type: "int",
        },
      ];
    case "value":
      return [valueLiteral(entries[0]?.[0])];
    case "values":
      return [
        {
          kind: "list",
          items: entries.map((e) => valueLiteral(e[0])),
          of: "values",
        },
      ];
    case "strings":
      return [
        {
          kind: "list",
          items: entries.map((e) => ({
            kind: "string",
            value: e[0]?.value ?? "",
          })),
        },
      ];
    case "kv":
      // Like the builder: `values()` gives each entry its own element,
      // `*_uniform()` sets all entries on every element.
      return [
        {
          kind: "list",
          items: step.type.endsWith("_uniform")
            ? entries.map(pairOf)
            : entries.map((entry) => ({
                kind: "list",
                items: [pairOf(entry)],
              })),
        },
      ];
    case "orders":
      return [
        oneOrList(
          entries.map((entry) => ({
            kind: "order",
            direction: entry[0]?.selectedOption === "desc" ? "desc" : "asc",
            key: valueLiteral(entry[1]),
          })),
        ),
      ];
    /* v8 ignore next 2 -- @preserve: steps without arguments have no args */
    default:
      return [];
  }
};

export const stepsToChain = (steps: (QueryStep | CopiedStep)[]): ChainCall[] =>
  steps.map((step) => ({
    method: getQueryStepLabel(step.type),
    args: stepArgs(step),
  }));

// --- chain -> steps -------------------------------------------------------

class ChainError extends Error {}

const fail = (message: string): never => {
  throw new ChainError(message);
};

const describe = (literal: Literal): string =>
  literal.kind === "number" ? `number ${literal.value}` : literal.kind;

const scalarField = (literal: Literal): QueryStepFieldValue => {
  switch (literal.kind) {
    case "string":
      return { selectedOption: "string", value: literal.value };
    case "boolean":
      return { selectedOption: "boolean", value: String(literal.value) };
    case "number":
      return {
        selectedOption: literal.type === "int" ? "signed" : literal.type,
        value: literal.value,
      };
    default:
      return fail(`Expected a value, got a ${describe(literal)}`);
  }
};

const valueField = (literal: Literal): QueryStepFieldValue => {
  if (literal.kind !== "array" && literal.kind !== "list") {
    return scalarField(literal);
  }
  // Untyped integers take the type of the other numbers, e.g. `vec![1u32, 2]`.
  const typed = literal.items.find(
    (item) => item.kind === "number" && item.type !== "int",
  );
  const fields = literal.items.map((item) =>
    item.kind === "number" && item.type === "int" && typed?.kind === "number"
      ? { selectedOption: typed.type, value: item.value }
      : scalarField(item),
  );
  const options = new Set(fields.map((field) => field.selectedOption));
  if (options.size > 1) fail("Array values must have one type");
  const [option = "string"] = options;
  if (option === "string" && fields.some((f) => f.value?.includes(","))) {
    fail("Strings in an array value cannot contain commas");
  }
  return {
    selectedOption: `${option}[]`,
    value: fields.map((field) => field.value).join(", "),
  };
};

const checkOption = (
  type: QueryType,
  field: QueryStepFieldValue,
  index = 0,
): QueryStepFieldValue => {
  const options = allowedOptions(type, index);
  if (!options.includes(field.selectedOption)) {
    fail(
      `${getQueryStepLabel(type)}() takes ${options.join(" or ")}, not ${field.selectedOption}`,
    );
  }
  return field;
};

const items = (literal: Literal): Literal[] =>
  literal.kind === "list" || literal.kind === "array"
    ? literal.items
    : [literal];

const integer = (literal: Literal, what: string): string => {
  if (
    literal.kind !== "number" ||
    literal.type === "float" ||
    !/^-?\d+$/.test(literal.value)
  ) {
    return fail(`Expected ${what}, got a ${describe(literal)}`);
  }
  return literal.value;
};

const idField = (literal: Literal): QueryStepFieldValue =>
  literal.kind === "string"
    ? { selectedOption: "string", value: literal.value }
    : { selectedOption: "signed", value: integer(literal, "an id") };

const pairEntry = (type: QueryType, literal: Literal): QueryStepArgEntry => {
  if (literal.kind !== "pair") {
    return fail(`Expected a (key, value) pair, got a ${describe(literal)}`);
  }
  return [
    checkOption(type, valueField(literal.key), 0),
    checkOption(type, valueField(literal.value), 1),
  ];
};

const toArgs = (
  type: QueryType,
  args: Literal[],
): QueryStepArgEntry[] | undefined => {
  const shape = argShape(type);
  const label = getQueryStepLabel(type);
  if (shape === "none" && args.length) fail(`${label}() takes no arguments`);
  if (args.length > 1) fail(`${label}() takes one argument`);
  const [arg] = args;
  if (!arg) return undefined;
  switch (shape) {
    case "ids":
      return items(arg).map((item) => [idField(item)]);
    case "id":
      return [[idField(arg)]];
    case "count": {
      const value = integer(arg, "a count");
      if (value.startsWith("-")) fail(`${label}() takes a count of 0 or more`);
      return [[{ selectedOption: "unsigned", value }]];
    }
    case "value":
      return [[checkOption(type, valueField(arg))]];
    case "values":
    case "strings":
      return items(arg).map((item) => [checkOption(type, valueField(item))]);
    case "kv":
      if (type.endsWith("_uniform")) {
        return items(arg).map((item) => pairEntry(type, item));
      }
      return items(arg).map((element) => {
        const kvs = element.kind === "pair" ? [element] : items(element);
        if (kvs.length !== 1 || !kvs[0]) {
          return fail(
            `The builder's ${label}() sets one key-value per element; use ${label}_uniform() for several`,
          );
        }
        return pairEntry(type, kvs[0]);
      });
    default:
      return items(arg).map((order) => {
        if (order.kind !== "order") {
          return fail(`Expected an ordering, got a ${describe(order)}`);
        }
        return [
          { selectedOption: order.direction },
          checkOption(type, valueField(order.key), 1),
        ];
      });
  }
};

/** `distance(LessThan(3))` -> `distance_less_than(3)`, `value(Equal(1))` -> `value(1)`. */
const COMPARISON_METHODS = new Set([
  "distance",
  "edge_count",
  "edge_count_from",
  "edge_count_to",
  "value",
]);

const normalize = ({ method, args }: ChainCall): ChainCall => {
  const [arg] = args;
  if (!COMPARISON_METHODS.has(method) || arg?.kind !== "comparison") {
    return { method, args };
  }
  if (arg.op === "equal") return { method, args: [arg.value] };
  return {
    method: method === "value" ? arg.op : `${method}_${arg.op}`,
    args: [arg.value],
  };
};

const ROOTS = [
  ...new Set([
    /* v8 ignore next 2 -- @preserve: both roots exist */
    ...(queryApiMock.exec?.followers ?? []),
    ...(queryApiMock.exec_mut?.followers ?? []),
  ]),
];

/**
 * Turns a chain into builder steps. Steps are matched by method name from
 * any query root, so a chain is accepted on any tab and validated there.
 */
export const chainToSteps = (
  chain: ChainCall[],
): { steps: CopiedStep[] } | { error: string } => {
  // The builder terminates queries itself.
  const calls = chain.at(-1)?.method === "query" ? chain.slice(0, -1) : chain;
  if (!calls.length) return { error: "The query has no steps" };
  try {
    let candidates = ROOTS;
    let previous: string | undefined;
    const steps = calls.map((call) => {
      const { method, args } = normalize(call);
      const type = candidates.find(
        (candidate) => getQueryStepLabel(candidate) === method,
      );
      if (!type) {
        return fail(
          previous
            ? `${method}() cannot follow ${previous}()`
            : `A query cannot start with ${method}()`,
        );
      }
      /* v8 ignore next -- @preserve: every follower is a known step */
      candidates = queryApiMock[type]?.followers ?? [];
      previous = method;
      const stepArgs = toArgs(type, args);
      return stepArgs ? { type, args: stepArgs } : { type };
    });
    return { steps };
  } catch (error) {
    /* v8 ignore next -- @preserve: only ChainError is thrown here */
    if (!(error instanceof ChainError)) throw error;
    return { error: error.message };
  }
};
