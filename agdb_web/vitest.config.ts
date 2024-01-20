import { fileURLToPath, URL } from "node:url";
import { configDefaults, coverageConfigDefaults } from "vitest/config";
import { defineVitestConfig } from "@nuxt/test-utils/config";

export default defineVitestConfig({
    test: {
        environment: "nuxt",
        exclude: [...configDefaults.exclude, "e2e/*"],
        root: fileURLToPath(new URL("./", import.meta.url)),
        coverage: {
            provider: "v8",
            all: true,
            exclude: [
                ...coverageConfigDefaults.exclude,
                "e2e/*",
                "*.config.ts",
                "*/**/*.vue",
            ],
        },
    },
    build: {
        target: ["es2015", "edge88", "firefox78", "chrome87", "safari12"],
    },
    resolve: {
        alias: {
            "@": fileURLToPath(new URL(".", import.meta.url)),
        },
    },
});
