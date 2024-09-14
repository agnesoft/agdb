import { describe, it, expect, vi } from "vitest";
import { cleanup, renderHook } from "@testing-library/react";
import { useHighlight } from "./hooks";
import { render } from "@testing-library/react";
import { beforeEach } from "node:test";

const { languagePackMock } = vi.hoisted(() => ({
    languagePackMock: vi.fn(),
    highlightModuleMock: vi.fn(),
}));

vi.mock("highlight.js/lib/core");

vi.mock("highlight.js/lib/languages/json", () => languagePackMock);
vi.mock("highlight.js/lib/languages/rust", () => languagePackMock);
vi.mock("highlight.js/lib/languages/python", () => languagePackMock);
vi.mock("highlight.js/lib/languages/php", () => languagePackMock);
vi.mock("highlight.js/lib/languages/javascript", () => languagePackMock);
vi.mock("highlight.js/lib/languages/typescript", () => languagePackMock);

describe("useHighlight", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.resetAllMocks();
        vi.resetModules();
    });
    describe("highlight", () => {
        beforeEach(() => {
            vi.clearAllMocks();
            vi.resetModules();
        });
        it("should highlight the code", async () => {
            const highlightElementMock = vi.fn();
            const hljs = await import("highlight.js/lib/core");
            hljs.default.highlightElement = highlightElementMock;

            const code = (
                <div>{`{
                    "name": "John Doe",
                    "age": 30,
                    "email": "
                }`}</div>
            );
            const { container } = render(code);
            const { result } = renderHook(() => useHighlight());
            result.current.highlight(container.firstChild as HTMLElement);
            expect(highlightElementMock).toHaveBeenCalledWith(
                container.firstChild,
            );
        });

        it("should not highlight the code again", async () => {
            const highlightElementMock = vi.fn();
            const hljs = await import("highlight.js/lib/core");
            hljs.default.highlightElement = highlightElementMock;

            const code = (
                <div>{`{
                    "name": "John Doe",
                    "age": 30,
                    "email": "
                }`}</div>
            );
            const { container } = render(code);
            if (container.firstChild)
                (container.firstChild as HTMLElement).dataset.highlighted =
                    "true";
            const { result } = renderHook(() => useHighlight());
            result.current.highlight(container.firstChild as HTMLElement);
            expect(highlightElementMock).not.toHaveBeenCalled();
        });
    });

    describe("setLanguage", () => {
        beforeEach(() => {
            vi.clearAllMocks();
            vi.resetAllMocks();
            vi.resetModules();
            cleanup();
        });

        it.each([
            ["json"],
            ["rust"],
            ["python"],
            ["php"],
            ["javascript"],
            ["typescript"],
        ])("should set the language - %s", async (language) => {
            const registerLanguageMock = vi.fn();
            const hljs = await import("highlight.js/lib/core");
            hljs.default.registerLanguage = registerLanguageMock;
            const { result } = renderHook(() => useHighlight());
            result.current.setLanguage(language);
            expect(registerLanguageMock).toHaveBeenCalledOnce();
        });

        it("should set the language - empty", async () => {
            const registerLanguageMock = vi.fn();
            const hljs = await import("highlight.js/lib/core");
            hljs.default.registerLanguage = registerLanguageMock;
            const { result } = renderHook(() => useHighlight());
            result.current.setLanguage("");
            expect(registerLanguageMock).not.toHaveBeenCalled();
        });
    });
});
