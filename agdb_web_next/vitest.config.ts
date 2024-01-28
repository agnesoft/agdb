import {
    defineConfig,
    coverageConfigDefaults,
    configDefaults,
} from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
    plugins: [react()],
    test: {
        environment: "jsdom",
        exclude: [...configDefaults.exclude, "e2e/*"],
        root: path.resolve(__dirname, "."),
        coverage: {
            provider: "v8",
            all: true,
            exclude: [
                ...coverageConfigDefaults.exclude,
                "e2e/*",
                "*.config.*",
                "middleware.ts",
            ],
        },
    },
    build: {
        target: ["es2015", "edge88", "firefox78", "chrome87", "safari12"],
    },
    resolve: {
        alias: {
            "@": path.resolve(__dirname, "."),
        },
    },
});
