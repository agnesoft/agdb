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

  it("emphasizes words in a text", () => {
    const content = convertArrayOfStringsToContent(["Next test Body"], {
      emphasizedWords: ["test"],
    });
    expect(content).toEqual([
      {
        paragraph: [
          {
            text: "Next ",
          },
          {
            text: "test",
            className: "emphasized",
          },
          {
            text: " Body",
          },
        ],
      },
    ]);
  });
});
