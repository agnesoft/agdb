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

/**
 * Whether pasted text should become a query rather than plain text, e.g.
 * when pasting into the step input: it must look like a method chain, so
 * pasting a single word such as an alias still pastes it as text.
 */
export const pastedQuery = (text: string): CopiedStep[] | undefined => {
  if (!/[.(]/.test(text)) return undefined;
  const result = parseSteps(text);
  return "steps" in result ? result.steps : undefined;
};

/**
 * Copies the steps as code to the system clipboard. Returns "local" when
 * that failed but the browser has no clipboard API to read it back anyway
 * (e.g. on plain http): the row is then kept for the paste button only.
 */
export const copySteps = async (
  steps: QueryStep[],
): Promise<"copied" | "local" | "failed"> => {
  const text = formatSteps(steps);
  const noReadApi = typeof navigator.clipboard?.readText !== "function";
  // The paste button cannot read the clipboard without the API.
  if (noReadApi) lastCopied = text;
  if (await copyText(text)) return "copied";
  return noReadApi ? "local" : "failed";
};

export type ReadResult = PasteResult | { denied: true };

/**
 * Reads steps from the system clipboard. The last row copied in this tab is
 * used only when the browser has no clipboard API (e.g. on plain http),
 * never when reading was denied, so a stale row is not pasted.
 */
export const readCopiedSteps = async (): Promise<ReadResult> => {
  if (typeof navigator.clipboard?.readText !== "function") {
    return parseSteps(lastCopied ?? "");
  }
  try {
    return parseSteps(await navigator.clipboard.readText());
  } catch {
    return { denied: true };
  }
};

/** For tests. */
export const resetCopiedSteps = (): void => {
  lastCopied = undefined;
};
