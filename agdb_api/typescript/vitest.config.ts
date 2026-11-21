import { defineConfig } from "vitest/config";

export default defineConfig({
    test: {
        coverage: {
            exclude: [
                "dist/**",
                "src/api.ts",
                "eslint.config.mjs",
                "src/index.ts",
                "src/openapi.d.ts",
                "src/query_builder.ts",
                "src/types.ts",
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
