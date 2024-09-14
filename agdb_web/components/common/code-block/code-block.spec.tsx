import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { CodeBlock } from "./code-block";

// vi.mock("next/router", () => ({
//     useRouter: () => ({
//         pathname: "/",
//         locale: "en-US",
//     }),
// }));
vi.mock("@/hooks/i18n", () => ({
    useI18n: () => ({
        t: (key: string) => {
            if (key === "button.copy-code") return "Copy code";
            return "";
        },
    }),
}));
describe("CodeBlock", () => {
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
});
