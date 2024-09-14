import { FC, useRef, useEffect } from "react";
import styles from "./code-block.module.scss";
import { CopyIcon } from "nextra/icons";
import { useI18n } from "@/hooks/i18n";
import { useHighlight } from "./hooks";

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
    const { t } = useI18n();

    const { highlight, setLanguage } = useHighlight();
    setLanguage(language);
    const codeRef = useRef(null);

    useEffect(() => {
        codeRef.current && highlight(codeRef.current);
    }, [code, highlight]);

    return (
        <div className={styles.codeBlock}>
            {header && <div className={styles.header}>{header}</div>}
            <div className={styles.wrapper}>
                <pre>
                    <code ref={codeRef} className={language}>
                        {code}
                    </code>
                </pre>
                {copy && (
                    <button
                        className={styles.copyButton}
                        onClick={() => navigator.clipboard.writeText(code)}
                        title={t("button.copy-code")}
                        data-testId="copy-code"
                    >
                        <CopyIcon />
                    </button>
                )}
            </div>
        </div>
    );
};
