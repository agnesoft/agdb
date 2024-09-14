import hljs from "highlight.js/lib/core";

export const useHighlight = () => {
    const highlight = (code: HTMLElement) => {
        return hljs.highlightElement(code);
    };

    const setLanguage = (language: string) => {
        switch (language) {
            case "json":
                hljs.registerLanguage(
                    "json",
                    require("highlight.js/lib/languages/json"),
                );
                break;
            case "rust":
                hljs.registerLanguage(
                    "rust",
                    require("highlight.js/lib/languages/rust"),
                );
                break;
            case "python":
                hljs.registerLanguage(
                    "python",
                    require("highlight.js/lib/languages/python"),
                );
                break;
            case "php":
                hljs.registerLanguage(
                    "php",
                    require("highlight.js/lib/languages/php"),
                );
                break;
            case "javascript":
                hljs.registerLanguage(
                    "javascript",
                    require("highlight.js/lib/languages/javascript"),
                );
                break;
            case "typescript":
                hljs.registerLanguage(
                    "typescript",
                    require("highlight.js/lib/languages/typescript"),
                );
                break;
            default:
                break;
        }
    };

    return { highlight, setLanguage };
};
