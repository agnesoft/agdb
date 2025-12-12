/** @type {import('next').NextConfig} */
import nextra from "nextra";

const withNextra = nextra({
    theme: "nextra-theme-docs",
    themeConfig: "./theme.config.tsx",
    defaultShowCopyCode: true,
});

export default withNextra({
    i18n: {
        locales: ["en-US", "cs-CZ"],
        defaultLocale: "en-US",
        localeDetection: false,
    },
});
