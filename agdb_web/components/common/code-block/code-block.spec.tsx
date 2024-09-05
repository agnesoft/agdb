import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { CodeBlock } from "./code-block";

describe("CodeBlock", () => {
    it("should render the code block with correct code and language", () => {
        const code = `{
            "name": "John Doe",
            "age": 30,
            "email": "
        }`;
        render(<CodeBlock code={code} language="json" />);
        const codeElement = screen.getByText(code);
        expect(codeElement).toBeDefined();
    });
});
