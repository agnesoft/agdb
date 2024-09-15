import { FC, useRef, useEffect, useState } from "react";
import styles from "./code-block.module.scss";
import { CopyIcon } from "nextra/icons";
import { useI18n } from "@/hooks/i18n";
import { useHighlight } from "./hooks";

export interface CodeBlockProps {
    code?: string;
    language: "json" | "javascript" | "typescript" | "rust" | "python" | "php";
    header?: string;
    copy?: boolean;
    showButtonText?: string;
    hideButtonText?: string;
    onLoad?: () => void;
}

export const CodeBlock: FC<CodeBlockProps> = ({
    code,
    language,
    header,
    copy = true,
    showButtonText = "Show code",
    hideButtonText = "Hide code",
    onLoad,
}) => {
    const { t } = useI18n();

    const { highlight, setLanguage } = useHighlight();

    const [hidden, setHidden] = useState(code ? false : true);

    setLanguage(language);
    const codeRef = useRef(null);

    useEffect(() => {
        codeRef.current && highlight(codeRef.current);
    }, [code, highlight]);

    const handleShowClick = () => {
        setHidden(false);
        !code && onLoad && onLoad();
    };

    return (
        <div className={styles.codeBlock}>
            {header && (
                <div className={styles.header}>
                    <span>{header}</span>
                    {onLoad && !hidden && (
                        <button
                            type="button"
                            data-testid="hide-code"
                            onClick={() => setHidden(true)}
                        >
                            {hideButtonText}
                        </button>
                    )}
                </div>
            )}
            <div className={styles.wrapper}>
                {code && !hidden ? (
                    <pre>
                        <code ref={codeRef} className={language}>
                            {code}
                        </code>
                    </pre>
                ) : (
                    onLoad && (
                        <div className={styles.buttonWrapper}>
                            <button
                                type="button"
                                className={styles.showButton}
                                onClick={handleShowClick}
                                data-testid="show-code"
                            >
                                {showButtonText}
                            </button>
                        </div>
                    )
                )}

                {code && copy && !hidden && (
                    <button
                        className={styles.copyButton}
                        onClick={() => navigator.clipboard.writeText(code)}
                        title={t("button.copy-code")}
                        data-testid="copy-code"
                    >
                        <CopyIcon />
                    </button>
                )}
            </div>
        </div>
    );
};
