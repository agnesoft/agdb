import { DocsThemeConfig, useConfig } from "nextra-theme-docs";
import Logo from "@/components/layout/logo";
import Footer from "@/components/layout/footer";
import { useRouter } from "nextra/hooks";

const useHead = () => {
    const { asPath, defaultLocale, locale } = useRouter();

    const config = useConfig();

    const url =
        "https://my-app.com" +
        (defaultLocale === locale ? asPath : `/${locale}${asPath}`);
    const title = config.title ? `${config.title} | agdb` : "agdb";

    return (
        <>
            <title>{title}</title>
            <meta property="og:url" content={url} />
            <meta property="og:title" content={title} />
            <meta
                property="og:description"
                content={config.frontMatter.description || "agdb docs"}
            />
        </>
    );
};

const config: DocsThemeConfig = {
    logo: Logo,
    project: {
        link: "https://github.com/agnesoft/agdb",
    },
    // chat: {
    //     link: "https://discord.com",
    // },
    docsRepositoryBase: "https://github.com/agnesoft/agdb/blob/main/agdb_web",
    footer: { content: Footer },
    i18n: [
        { locale: "en-US", name: "English" },
        { locale: "cs-CZ", name: "Čeština" },
    ],
    head: useHead,
    // banner: {
    //   key: '2.0-release',
    //   text: (
    //     <a href="/release" target="_blank">
    //       🎉 agdb 2.0 is released. Read more →
    //     </a>
    //   )
    // }
};

export default config;
