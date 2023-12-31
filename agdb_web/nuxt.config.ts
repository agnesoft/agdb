import { fileURLToPath, URL } from "node:url";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    devServer: {
        port: 4000,
    },
    devtools: { enabled: true },
    modules: ["@nuxt/content", "@nuxt/test-utils/module"],
    vite: {
        resolve: {
            alias: {
                "@": fileURLToPath(new URL(".", import.meta.url)),
            },
        },
        build: {
            target: ["es2015", "edge88", "firefox78", "chrome87", "safari12"],
        },
    },
});
