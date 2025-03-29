import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config.mjs";
import vitestShared from "@agdb-studio/config/vitest.shared";
import path from "path";

export default mergeConfig(
  mergeConfig(viteConfig, vitestShared),
  defineConfig({
    test: {
      root: path.resolve(__dirname, "."),
      setupFiles: ["./vitest.setup.ts"],
    },
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
  }),
);
