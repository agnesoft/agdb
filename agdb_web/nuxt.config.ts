import { resolve } from "node:path";
import { fileURLToPath, URL } from "node:url";
// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    devServer: {
        port: 5001,
    },
    devtools: { enabled: true },
    modules: [
        [
            "@nuxt/content",
            {
                documentDriven: { injectPage: false },
                sources: {
                    en: {
                        base: resolve(__dirname, "content/en"),
                        driver: "fs",
                        prefix: "/",
                    },
                },
                highlight: {
                    theme: {
                        default: "github-light",
                        dark: "github-dark",
                        sepia: "monokai",
                    },
                },
            },
        ],
        "@nuxt/test-utils/module",
    ],
    vite: {
        resolve: {
            alias: {
                "@": fileURLToPath(new URL(".", import.meta.url)),
            },
        },
        build: {
            target: ["es2015", "edge88", "firefox78", "chrome87", "safari12"],
        },
        css: {
            preprocessorOptions: {
                less: {
                    modifyVars: {
                        "@smallWidth": "600px",
                        "@mediumWidth": "768px",
                        "@largeWidth": "992px",
                    },
                },
            },
        },
    },
    css: ["@/assets/less/main.less"],
});
