import type { Content, Paragraph } from "./types";

export type ConvertParams = {
  emphasizedWords?: string[];
};

export const EMPHASIZED_CLASSNAME = "emphasized";

const emphasizeWords = (text: string, words: string[]): Paragraph[] => {
  const parts = text.split(new RegExp(`(${words.join("|")})`, "g"));

  return parts.map((part) => {
    if (words.includes(part)) {
      return { text: part, className: EMPHASIZED_CLASSNAME };
    }
    return { text: part };
  });
};

const convertArrayOfStringsToContent = (
  array: string[],
  params: ConvertParams | undefined = undefined,
): Content[] => {
  return array.map((text) => {
    if (params?.emphasizedWords) {
      return { paragraph: emphasizeWords(text, params.emphasizedWords) };
    }
    return { paragraph: [{ text }] };
  });
};

export { convertArrayOfStringsToContent };
