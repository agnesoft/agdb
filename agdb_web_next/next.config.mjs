/** @type {import('next').NextConfig} */
import withNextra from "nextra";
import createNextIntlPlugin from "next-intl/plugin";

const withNextIntl = createNextIntlPlugin();

const nextConfig = {
    i18n: {
        locales: ["en-US", "cs-CZ"],
        defaultLocale: "en-US",
        localeDetection: false,
    },
};

export default withNextra({
    theme: "nextra-theme-docs",
    themeConfig: "./theme.config.tsx",
})(withNextIntl(nextConfig));
