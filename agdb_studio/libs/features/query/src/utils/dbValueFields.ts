import { convertToDbValue, type Components } from "@agnesoft/agdb_api";
import type { QueryStepFieldValue } from "../composables/types";
import { VALUE_TYPES } from "../mock/queryApiMock";
import { parseDbValue } from "./queryExecutionUtils";

/** Edits raw bytes as comma-separated numbers 0-255 (editor-only type). */
export const BYTES_TYPE = "bytes";

/**
 * Marks a nested `VecDbValue`. It has no editor, so the panel keeps the
 * original value and shows it read-only.
 */
export const NESTED_TYPE = "nested";

/** Value types offered when editing element values. */
export const EDITABLE_VALUE_TYPES: readonly string[] = [
  ...VALUE_TYPES,
  BYTES_TYPE,
];

const isBooleanString = (value: string): boolean =>
  value === "true" || value === "false";

const splitItems = (raw: string | undefined): string[] =>
  (raw ?? "")
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);

/** Converts a typed form field (type option + raw text) to a `DbValue`. */
export const fieldToDbValue = (
  field: QueryStepFieldValue,
): Components.Schemas.DbValue => {
  if (field.selectedOption === BYTES_TYPE) {
    return { Bytes: splitItems(field.value).map(Number) };
  }
  const parsed = parseDbValue(field);
  // An empty array would otherwise be inferred as `VecI64`.
  if (
    (field.selectedOption === "string[]" ||
      field.selectedOption === "boolean[]") &&
    Array.isArray(parsed)
  ) {
    return { VecString: parsed.map(String) };
  }
  return convertToDbValue(parsed);
};

/**
 * Converts a `DbValue` into a form field (type option + raw text) so that it
 * can be pre-filled in an editor. `fieldToDbValue` converts it back to the
 * same variant (nested values excepted, see `NESTED_TYPE`).
 */
export const dbValueToField = (
  value: Components.Schemas.DbValue,
): QueryStepFieldValue => {
  // Booleans are stored as strings, so strings always stay strings.
  if ("String" in value) {
    return { selectedOption: "string", value: value.String };
  }
  if ("I64" in value) {
    return { selectedOption: "signed", value: String(value.I64) };
  }
  if ("U64" in value) {
    return { selectedOption: "unsigned", value: String(value.U64) };
  }
  if ("F64" in value) {
    return { selectedOption: "float", value: String(value.F64) };
  }
  if ("VecI64" in value) {
    return { selectedOption: "signed[]", value: value.VecI64.join(", ") };
  }
  if ("VecU64" in value) {
    return { selectedOption: "unsigned[]", value: value.VecU64.join(", ") };
  }
  if ("VecF64" in value) {
    return { selectedOption: "float[]", value: value.VecF64.join(", ") };
  }
  if ("VecString" in value) {
    return { selectedOption: "string[]", value: value.VecString.join(", ") };
  }
  if ("Bytes" in value) {
    return { selectedOption: BYTES_TYPE, value: value.Bytes.join(", ") };
  }
  return {
    selectedOption: NESTED_TYPE,
    value: value.VecDbValue.map((v) => dbValueToField(v).value).join(", "),
  };
};

/** Short human-readable type label of a `DbValue`, e.g. "i64" or "string[]". */
export const dbValueTypeLabel = (value: Components.Schemas.DbValue): string => {
  const key = Object.keys(value)[0] ?? "";
  const labels: Record<string, string> = {
    String: "string",
    I64: "i64",
    U64: "u64",
    F64: "f64",
    Bytes: "bytes",
    VecI64: "i64[]",
    VecU64: "u64[]",
    VecF64: "f64[]",
    VecString: "string[]",
    VecDbValue: "value[]",
  };
  return labels[key] ?? key;
};

const SIGNED = /^-?\d+$/;
const UNSIGNED = /^\d+$/;

/** Integer text that is well-formed and exactly representable in JavaScript. */
const isInteger = (item: string, pattern: RegExp): boolean =>
  pattern.test(item) && Number.isSafeInteger(Number(item));

const isFiniteNumber = (item: string): boolean =>
  item.length > 0 && Number.isFinite(Number(item));

const isByte = (item: string): boolean =>
  UNSIGNED.test(item) && Number(item) <= 255;

const PRECISION_HINT = `(at most ${Number.MAX_SAFE_INTEGER} in absolute value)`;

/** Returns an error message if the field cannot be converted, otherwise undefined. */
export const validateField = (
  field: QueryStepFieldValue,
): string | undefined => {
  const raw = (field.value ?? "").trim();
  const items = splitItems(raw);
  switch (field.selectedOption) {
    case "signed":
      return isInteger(raw, SIGNED)
        ? undefined
        : `Expected an integer ${PRECISION_HINT}`;
    case "unsigned":
      return isInteger(raw, UNSIGNED)
        ? undefined
        : `Expected a non-negative integer ${PRECISION_HINT}`;
    case "float":
      return isFiniteNumber(raw) ? undefined : "Expected a finite number";
    case "boolean":
      return isBooleanString(raw.toLowerCase())
        ? undefined
        : "Expected true or false";
    case "signed[]":
      return items.every((item) => isInteger(item, SIGNED))
        ? undefined
        : `Expected comma-separated integers ${PRECISION_HINT}`;
    case "unsigned[]":
      return items.every((item) => isInteger(item, UNSIGNED))
        ? undefined
        : `Expected comma-separated non-negative integers ${PRECISION_HINT}`;
    case "float[]":
      return items.every(isFiniteNumber)
        ? undefined
        : "Expected comma-separated finite numbers";
    case "boolean[]":
      return items.every((item) => isBooleanString(item.toLowerCase()))
        ? undefined
        : "Expected comma-separated true/false values";
    case BYTES_TYPE:
      return items.every(isByte)
        ? undefined
        : "Expected comma-separated bytes (0-255)";
    default:
      return undefined;
  }
};
