import { DocsThemeConfig } from "nextra-theme-docs";
import Logo from "@/components/layout/logo";

const config: DocsThemeConfig = {
    logo: Logo,
    project: {
        link: "https://github.com/agnesoft/agdb",
    },
    // chat: {
    //     link: "https://discord.com",
    // },
    docsRepositoryBase: "https://github.com/agnesoft/agdb",
    footer: {
        text: "Copyright @ 2024 agdb",
    },
    i18n: [
        { locale: "en-US", text: "English" },
        { locale: "cs-CZ", text: "Čeština" },
    ],
};

export default config;
