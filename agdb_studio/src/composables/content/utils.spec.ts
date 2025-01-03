import { describe, it, expect } from "vitest";
import { convertArrayOfStringsToContent } from "@/composables/content/utils";

describe("Content utils", () => {
    it("converts an array of strings to content", () => {
        const content = convertArrayOfStringsToContent([
            "Test Body",
            "Test Body 2",
        ]);
        expect(content).toEqual([
            {
                paragraph: [
                    {
                        text: "Test Body",
                    },
                ],
            },
            {
                paragraph: [
                    {
                        text: "Test Body 2",
                    },
                ],
            },
        ]);
    });

    it("emphesizes words in a text", () => {
        const content = convertArrayOfStringsToContent(["Next test Body"], {
            emphesizedWords: ["test"],
        });
        expect(content).toEqual([
            {
                paragraph: [
                    {
                        text: "Next ",
                    },
                    {
                        text: "test",
                        className: "emphesized",
                    },
                    {
                        text: " Body",
                    },
                ],
            },
        ]);
    });
});