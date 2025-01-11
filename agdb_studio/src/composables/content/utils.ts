export type ConvertParams = {
    emphesizedWords?: string[];
};

export const EMPHESIZED_CLASSNAME = "emphesized";

const emphesizeWords = (text: string, words: string[]): Paragraph[] => {
    const parts = text.split(new RegExp(`(${words.join("|")})`, "g"));

    return parts.map((part) => {
        if (words.includes(part)) {
            return { text: part, className: EMPHESIZED_CLASSNAME };
        }
        return { text: part };
    });
};

const convertArrayOfStringsToContent = (
    array: string[],
    params: ConvertParams | undefined = undefined,
): Content[] => {
    return array.map((text) => {
        if (params?.emphesizedWords) {
            return { paragraph: emphesizeWords(text, params.emphesizedWords) };
        }
        return { paragraph: [{ text }] };
    });
};

export { convertArrayOfStringsToContent };
