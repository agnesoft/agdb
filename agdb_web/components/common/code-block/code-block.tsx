import { FC, useRef, useEffect } from "react";
import hljs from "highlight.js/lib/core";
import json from "highlight.js/lib/languages/json";
import rust from "highlight.js/lib/languages/rust";
import python from "highlight.js/lib/languages/python";
import php from "highlight.js/lib/languages/php";
import javascript from "highlight.js/lib/languages/javascript";
import typescript from "highlight.js/lib/languages/typescript";
import styles from "./code-block.module.scss";
import { CopyIcon } from "nextra/icons";

export interface CodeBlockProps {
    code: string;
    language: "json" | "javascript" | "typescript" | "rust" | "python" | "php";
    header?: string;
    copy?: boolean;
}

export const CodeBlock: FC<CodeBlockProps> = ({
    code,
    language,
    header,
    copy = true,
}) => {
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
    }
    const codeRef = useRef(null);

    useEffect(() => {
        if (codeRef.current) {
            hljs.highlightElement(codeRef.current);
        }
    }, [code]);

    return (
        <div>
            {header && <div className={styles.header}>{header}</div>}
            <pre>
                <code ref={codeRef} className={language}>
                    {code}
                </code>
            </pre>
            {copy && (
                <button
                    className={styles.copyButton}
                    onClick={() => navigator.clipboard.writeText(code)}
                >
                    <CopyIcon />
                </button>
            )}
        </div>
    );
};
