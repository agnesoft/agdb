import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import json from "highlight.js/lib/languages/json";
import python from "highlight.js/lib/languages/python";
import php from "highlight.js/lib/languages/php";
import javascript from "highlight.js/lib/languages/javascript";
import typescript from "highlight.js/lib/languages/typescript";

export const useHighlight = () => {
    const highlight = (code: HTMLElement) => {
        if (!code.dataset.highlighted) {
            hljs.highlightElement(code);
        }
    };

    const setLanguage = (language: string) => {
        switch (language) {
            case "json":
                hljs.registerLanguage("json", json);
                break;
            case "rust":
                hljs.registerLanguage("rust", rust);
                break;
            case "python":
                hljs.registerLanguage("python", python);
                break;
            case "php":
                hljs.registerLanguage("php", php);
                break;
            case "javascript":
                hljs.registerLanguage("javascript", javascript);
                break;
            case "typescript":
                hljs.registerLanguage("typescript", typescript);
                break;
            default:
                break;
        }
    };

    return { highlight, setLanguage };
};
