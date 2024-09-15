import { describe, it, expect, vi, beforeEach } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
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
        const component = render(
            <CodeBlock code={code} language="json" header="header" />,
        );
        const text = component.getByText('"John Doe"');
        expect(text).toBeDefined();
        const copyButton = component.getByTitle("Copy code");
        expect(copyButton).toBeDefined();
        const header = component.getByText("header");
        expect(header).toBeDefined();
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

    it("should show and hide the code block", () => {
        const code = `{
            "name": "John Doe",
            "age": 30,
            "email": "
        }`;

        const codeMock: {
            current: string | undefined;
        } = {
            current: undefined,
        };

        const component = render(
            <CodeBlock
                header="header"
                code={codeMock.current}
                language="json"
                onLoad={() => {
                    codeMock.current = code;
                }}
            />,
        );
        const showButton = component.getByText("Show code");
        fireEvent.click(showButton);

        const hideButton = component.getByText("Hide code");
        expect(hideButton).toBeDefined();
        fireEvent.click(hideButton);
        const showButtonAfterHide = component.getByText("Show code");
        expect(showButtonAfterHide).toBeDefined();
    });
});
