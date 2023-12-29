import { fileURLToPath } from "node:url";
import { mergeConfig, defineConfig, configDefaults } from "vitest/config";
import viteConfig from "./vite.config";
import path from "path";

export default mergeConfig(
    viteConfig,
    defineConfig({
        test: {
            environment: "jsdom",
            exclude: [...configDefaults.exclude, "e2e/*"],
            root: fileURLToPath(new URL("./", import.meta.url)),
            coverage: {
                all: true,
                exclude: [
                    "playwright.config.ts",
                    ".eslintrc.cjs",
                    "env.d.ts",
                    "src/openapi/schema.d.ts",
                ],
            },
        },
        resolve: {
            alias: {
                "@": path.resolve(__dirname, "./src"),
            },
        },
    }),
);
