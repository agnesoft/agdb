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
            provider: "istanbul",
            all: true,
            exclude: [
                ...coverageConfigDefaults.exclude,
                "e2e/*",
                "*.config.*",
                "middleware.ts",
            ],
            // reporter: ["text", ["html", { subdir: "coverage" }]],
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
