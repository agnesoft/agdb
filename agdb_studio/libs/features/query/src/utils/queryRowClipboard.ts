import type { CopiedStep, QueryStep } from "../composables/types";
import { chainToSteps, stepsToChain, type QueryLanguage } from "./queryChain";
import { rustLanguage } from "./queryLanguageRust";
import { copyText } from "./clipboard";

/**
 * Languages queries are copied to and pasted from. Copying uses the first;
 * pasting tries each in turn. Add more (e.g. TypeScript) here.
 */
export const QUERY_LANGUAGES: readonly QueryLanguage[] = [rustLanguage];

/** Rows are kept here too, for browsers without a clipboard API. */
let lastCopied: string | undefined;

export const formatSteps = (
  steps: (QueryStep | CopiedStep)[],
  /* v8 ignore next -- @preserve: one language for now */
  language: QueryLanguage = QUERY_LANGUAGES[0] ?? rustLanguage,
): string => language.format(stepsToChain(steps));

export type PasteResult = { steps: CopiedStep[] } | { error: string };

/** Reads steps from code in any of the supported languages. */
export const parseSteps = (text: string): PasteResult => {
  if (!text.trim()) return { error: "The clipboard does not contain a query" };
  let firstError: string | undefined;
  for (const language of QUERY_LANGUAGES) {
    const parsed = language.parse(text);
    const result =
      "calls" in parsed ? chainToSteps(parsed.calls) : { error: parsed.error };
    if ("steps" in result) return result;
    firstError ??= result.error;
  }
  /* v8 ignore next -- @preserve: there is always at least one language */
  return { error: firstError ?? "Unsupported query" };
};

/** Formats several rows, one chain per line. */
export const formatRows = (rows: (QueryStep | CopiedStep)[][]): string =>
  rows
    .filter((steps) => steps.length > 0)
    .map((steps) => formatSteps(steps))
    .join("\n");

export type PasteRowsResult = { rows: CopiedStep[][] } | { error: string };

const MARKER = "QueryBuilder::";

/**
 * Where the chains in the text start: each `QueryBuilder::` outside string
 * literals and at the outermost bracket depth any of them has, so a query
 * nested in another (e.g. `.from(QueryBuilder::search()...)`) does not
 * split its chain; `vec![...]` around the chains still works.
 */
const chainStarts = (text: string): number[] => {
  const found: { index: number; depth: number }[] = [];
  let depth = 0;
  let inString = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (inString) {
      if (c === "\\") i++;
      else if (c === '"') inString = false;
    } else if (c === '"') inString = true;
    else if (c === "(" || c === "[" || c === "{") depth++;
    else if (c === ")" || c === "]" || c === "}") depth--;
    else if (text.startsWith(MARKER, i)) {
      found.push({ index: i, depth });
      i += MARKER.length - 1;
    }
  }
  const outermost = Math.min(...found.map((f) => f.depth));
  return found.filter((f) => f.depth === outermost).map((f) => f.index);
};

/**
 * Reads one or more rows: the text is split at each top-level `QueryBuilder::` so
 * several chains (one per line, in a `vec![...]`, inside `db.exec(...)`)
 * each become a row. Text without the prefix is read as a single row.
 */
export const parseRows = (text: string): PasteRowsResult => {
  const starts = chainStarts(text);
  if (starts.length < 2) {
    const single = parseSteps(text);
    return "steps" in single ? { rows: [single.steps] } : single;
  }
  const rows: CopiedStep[][] = [];
  for (const [n, start] of starts.entries()) {
    const result = parseSteps(text.slice(start, starts[n + 1]));
    if ("error" in result) return { error: `Query ${n + 1}: ${result.error}` };
    rows.push(result.steps);
  }
  return { rows };
};

/**
 * Whether pasted text should become a query rather than plain text, e.g.
 * when pasting into the step input: it must look like a method chain, so
 * pasting a single word such as an alias still pastes it as text.
 */
export const pastedQuery = (text: string): CopiedStep[][] | undefined => {
  if (!/[.(]/.test(text)) return undefined;
  const result = parseRows(text);
  return "rows" in result ? result.rows : undefined;
};

/**
 * Copies the steps as code to the system clipboard. Returns "local" when
 * that failed but the browser has no clipboard API to read it back anyway
 * (e.g. on plain http): the row is then kept for the paste button only.
 */
export const copySteps = async (
  steps: QueryStep[],
): Promise<"copied" | "local" | "failed"> => copyRows([steps]);

/** Copies several rows as code, one chain per line. */
export const copyRows = async (
  rows: QueryStep[][],
): Promise<"copied" | "local" | "failed"> => {
  const text = formatRows(rows);
  const noReadApi = typeof navigator.clipboard?.readText !== "function";
  // The paste button cannot read the clipboard without the API.
  if (noReadApi) lastCopied = text;
  if (await copyText(text)) return "copied";
  return noReadApi ? "local" : "failed";
};

export type ReadResult = PasteRowsResult | { denied: true };

/**
 * Reads rows from the system clipboard. The last rows copied in this tab are
 * used only when the browser has no clipboard API (e.g. on plain http),
 * never when reading was denied, so stale rows are not pasted.
 */
export const readCopiedRows = async (): Promise<ReadResult> => {
  if (typeof navigator.clipboard?.readText !== "function") {
    return parseRows(lastCopied ?? "");
  }
  try {
    return parseRows(await navigator.clipboard.readText());
  } catch {
    return { denied: true };
  }
};

/** For tests. */
export const resetCopiedSteps = (): void => {
  lastCopied = undefined;
};
