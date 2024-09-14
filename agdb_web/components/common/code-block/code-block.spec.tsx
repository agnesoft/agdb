import { describe, it, expect, vi, beforeEach } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { CodeBlock } from "./code-block";

const writeText = vi.fn();

Object.assign(navigator, {
    clipboard: {
        writeText,
    },
});
vi.mock("@/hooks/i18n", () => ({
    useI18n: () => ({
        t: (key: string) => {
            if (key === "button.copy-code") return "Copy code";
            return "";
        },
    }),
}));
describe("CodeBlock", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        cleanup();
    });
    it("should render the code block with correct code and language", () => {
        const code = `{
            "name": "John Doe",
            "age": 30,
            "email": "
        }`;
        render(<CodeBlock code={code} language="json" />);
        const text = screen.getByText('"John Doe"');
        expect(text).toBeDefined();
        const copyButton = screen.getByTitle("Copy code");
        expect(copyButton).toBeDefined();
    });
    it("should copy the code to clipboard", () => {
        const code = `{
            "name": "John Doe",
            "age": 30,
            "email": "
        }`;
        const component = render(<CodeBlock code={code} language="json" />);
        const copyButton = component.queryByTestId("copy-code");
        copyButton?.click();
        expect(writeText).toHaveBeenCalledWith(code);
    });
});
