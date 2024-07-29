import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { DEFAULT_LOCALE } from "nextra/constants";

type MessagesStructure = { [key: string]: string | MessagesStructure };

const iterateMessages = (
    prefix: string | null,
    obj: MessagesStructure,
    map: Map<string, string>,
): void => {
    for (const [key, value] of Object.entries(obj)) {
        const keyName = prefix ? `${prefix}.${key}` : key;
        if (typeof value === "string") {
            map.set(keyName, value);
        } else {
            iterateMessages(keyName, value, map);
        }
    }
};

export const useI18n = () => {
    const { locale } = useRouter();
    const [fallbackMessages, setFallbackMessages] = useState(
        new Map<string, string>(),
    );
    const [messages, setMessages] = useState(new Map<string, string>());

    const defaultLocale = DEFAULT_LOCALE;

    useEffect(() => {
        import(`../messages/${defaultLocale}.json`)
            .then((data) => {
                const messages = new Map<string, string>();
                iterateMessages(null, data, messages);
                setFallbackMessages(messages);
            })
            .catch(() => setFallbackMessages(new Map<string, string>()));
    }, [defaultLocale]);

    useEffect(() => {
        import(`../messages/${locale}.json`)
            .then((data) => {
                const messages = new Map<string, string>();
                iterateMessages(null, data, messages);
                setMessages(messages);
            })
            .catch(() => setMessages(new Map<string, string>()));
    }, [locale]);

    const t = (key: string): string => {
        return messages.get(key) || fallbackMessages.get(key) || "";
    };

    return { locale, t };
};
