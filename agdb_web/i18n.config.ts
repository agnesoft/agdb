export default {
    defaultLocale: "en",
    locales: [
        {
            code: "en",
            iso: "en-US",
            file: "en-US.json",
            name: "English",
        },
        {
            code: "cs",
            iso: "cs-CZ",
            file: "cs-CZ.json",
            name: "Čeština",
        },
    ],
    langDir: "locales/",
    fallbackLocale: "en",
    strategy: "prefix_and_default",
};
