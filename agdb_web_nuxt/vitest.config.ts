import {
  defineConfig,
  configDefaults,
  coverageConfigDefaults,
} from "vitest/config";
import path from "path";
import { defineVitestProject } from "@nuxt/test-utils/config";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  test: {
    exclude: [...configDefaults.exclude, "e2e/*"],
    coverage: {
      provider: "v8",
      exclude: [
        ...coverageConfigDefaults.exclude,
        "e2e/*",
        "e2e-utils/*",
        "playwright-report/*",
        "*.config.*",
        "src/tests/*",
        "**/types.ts",
        "**/assets/**",
      ],
      thresholds: {
        lines: 100,
        functions: 100,
        branches: 100,
        statements: 100,
      },
      reporter: ["text", "html", "json"],
    },
    projects: [
      {
        test: {
          name: "unit",
          include: ["app/**/*.{test,spec}.ts"],
          environment: "jsdom",
          exclude: [
            ...coverageConfigDefaults.exclude,
            "**/*.nuxt.{test,spec}.ts",
          ],
        },
      },
      await defineVitestProject({
        test: {
          name: "nuxt",
          include: ["app/**/*.nuxt.{test,spec}.ts"],
          environment: "nuxt",
        },
      }),
    ],
    globals: true,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "."),
    },
  },
});
