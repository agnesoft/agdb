import { defineConfig, mergeConfig } from "vitest/config.js";
import viteConfig from "./vite.config.mjs";
import vitestShared from "@agdb-studio/testing/vitest.shared";
import path from "path";

export default mergeConfig(
  mergeConfig(viteConfig, vitestShared),
  defineConfig({
    test: {
      root: path.resolve(__dirname, "."),
      setupFiles: ["@agdb-studio/testing/vitest.setup.ts"],
    },
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
  }),
);
