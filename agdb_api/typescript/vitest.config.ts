import { defineConfig } from "vitest/config";

export default defineConfig({
    test: {
        coverage: {
            exclude: [
                "dist/**",
                "eslint.config.mjs",
                "src/openapi.d.ts",
                "query_test_generator.js",
                "tests/**",
                "vite.config.ts",
                "vitest.config.ts",
            ],
            thresholds: {
                lines: 100,
                functions: 100,
                branches: 95,
                statements: 100,
            },
            reporter: ["text", "html", "json"],
        },
    },
});
