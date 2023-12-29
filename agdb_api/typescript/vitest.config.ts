import { mergeConfig, defineConfig, configDefaults } from "vitest/config";
import viteConfig from "./vite.config";

export default mergeConfig(
    viteConfig,
    defineConfig({
        test: {
            coverage: {
                all: true,
                exclude: [".eslintrc.cjs", "src/schema.d.ts"],
            },
        },
    })
);
