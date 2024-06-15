import { mergeConfig, defineConfig } from "vitest/config";
import viteConfig from "./vite.config";

export default mergeConfig(
    viteConfig,
    defineConfig({
        test: {
            coverage: {
                all: true,
                exclude: [".eslintrc.cjs", "src/schema.d.ts"],
                thresholds: {
                    lines: 100,
                    functions: 100,
                    branches: 100,
                    statements: 100,
                },
            },
        },
    }),
);
