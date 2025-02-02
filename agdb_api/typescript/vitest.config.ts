import { mergeConfig, defineConfig } from "vitest/config";
import viteConfig from "./vite.config";

export default mergeConfig(
    viteConfig,
    defineConfig({
        test: {
            coverage: {
                all: true,
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
                    branches: 100,
                    statements: 100,
                },
                reporter: ["text", "html", "json"],
            },
        },
    }),
);
