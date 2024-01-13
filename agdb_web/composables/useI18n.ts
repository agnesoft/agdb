import { computed, ref } from "vue";
import localeData from "~/locales";
import i18nConfig from "~/i18n.config";

type Locale = {
    code: string;
    iso: string;
    name: string;
};

const localeCode = ref<string>(i18nConfig.defaultLocale);

const fallbackLocaleCode = ref<string>(i18nConfig.defaultLocale);

const locales: Locale[] = i18nConfig.locales;

const localeExists = (locale: string): boolean => {
    return locales.some((localeItem) => localeItem.code === locale);
};

const currentLocale = computed(() =>
    locales.find((locale) => locale.code === localeCode.value),
);

const fallbackLocale = computed(() =>
    locales.find((locale) => locale.code === fallbackLocaleCode.value),
);

const messages = ref(new Map<string, string>());

const fallbackMessages = ref(new Map<string, string>());

const currentPagePath = ref("/");

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

const loadMessages = (): void => {
    messages.value = new Map(fallbackMessages.value);
    if (!Object.prototype.hasOwnProperty.call(localeData, localeCode.value)) {
        return;
    }
    const localeMessage: MessagesStructure =
        localeData[localeCode.value as keyof typeof localeData];
    iterateMessages(null, localeMessage, messages.value);
};

const loadFallbackMessages = (): void => {
    if (
        !Object.prototype.hasOwnProperty.call(
            localeData,
            fallbackLocaleCode.value,
        )
    ) {
        return;
    }
    const fallbackLocaleMessages: MessagesStructure =
        localeData[fallbackLocaleCode.value as keyof typeof localeData];
    iterateMessages(null, fallbackLocaleMessages, fallbackMessages.value);
};

const t = (key: string): string => {
    return messages.value.get(key) || "";
};

const hasPathLocale = (path: string): boolean => {
    return locales.some(
        (locale) =>
            path.startsWith(`/${locale.code}/`) || path === `/${locale.code}`,
    );
};

const getLocalePath = (path: string): string => {
    const hasLocale = hasPathLocale(path);
    if (localeCode.value === fallbackLocale.value?.code) {
        const newPath = hasLocale
            ? path.replace(`/${localeCode.value}`, "")
            : path;
        return newPath === "" ? "/" : newPath;
    }
    return hasLocale ? path : `/${localeCode.value}${path}`;
};

const setLocaleCode = (code: string): void => {
    if (code === localeCode.value || !localeExists(code)) {
        return;
    }
    localeCode.value = code;
    loadMessages();
};

const setDefaultLocaleCode = (): void => {
    localeCode.value = fallbackLocale.value?.code || "en";
    messages.value = new Map(fallbackMessages.value);
};

const initI18n = (): void => {
    if (fallbackMessages.value.size > 0) return;
    loadFallbackMessages();
};

export default function useI18n() {
    return {
        localeCode,
        fallbackLocaleCode,
        localeExists,
        currentLocale,
        fallbackLocale,
        loadMessages,
        loadFallbackMessages,
        t,
        getLocalePath,
        setLocaleCode,
        setDefaultLocaleCode,
        currentPagePath,
        initI18n,
    };
}
