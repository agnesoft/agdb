import { useRouter } from "nextra/hooks";
import { useEffect, useState } from "react";

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
    const { locale, defaultLocale } = useRouter();
    const [fallbackMessages, setFallbackMessages] = useState(
        new Map<string, string>(),
    );
    const [messages, setMessages] = useState(new Map<string, string>());

    const processFallbackMessages = (data: MessagesStructure): void => {
        const messages = new Map<string, string>();
        iterateMessages(null, data, messages);
        setFallbackMessages(messages);
    };

    const processMessages = (data: MessagesStructure): void => {
        const messages = new Map<string, string>();
        iterateMessages(null, data, messages);
        setMessages(messages);
    };

    /* v8 ignore next -- @preserve */
    useEffect(() => {
        import(`../messages/${defaultLocale}.json`)
            .then(processFallbackMessages)
            .catch(() => setFallbackMessages(new Map<string, string>()));
    }, [defaultLocale]);

    useEffect(() => {
        import(`../messages/${locale}.json`)
            .then(processMessages)
            .catch(() => setMessages(new Map<string, string>()));
    }, [locale, defaultLocale]);

    const t = (key: string): string => {
        return messages.get(key) || fallbackMessages.get(key) || "";
    };

    return { locale, t };
};
