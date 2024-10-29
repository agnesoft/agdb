// import { fileURLToPath } from "node:url";
import {
    mergeConfig,
    defineConfig,
    configDefaults,
    coverageConfigDefaults,
} from "vitest/config";
import viteConfig from "./vite.config.mjs";
import path from "path";

export default mergeConfig(
    viteConfig,
    defineConfig({
        test: {
            environment: "jsdom",
            exclude: [...configDefaults.exclude, "e2e/*"],
            root: path.resolve(__dirname, "."),
            // root: fileURLToPath(new URL("./", import.meta.url)),
            coverage: {
                provider: "istanbul",
                all: true,
                exclude: [
                    ...coverageConfigDefaults.exclude,
                    "e2e/*",
                    "*.config.*",
                    "./src/main.ts",
                    "./src/App.vue",
                ],
                thresholds: {
                    lines: 100,
                    functions: 100,
                    branches: 100,
                    statements: 100,
                },
            },
            setupFiles: ["./vitest.setup.ts"],
            globals: true,
        },
        resolve: {
            alias: {
                "@": path.resolve(__dirname, "./src"),
            },
        },
    }),
);
