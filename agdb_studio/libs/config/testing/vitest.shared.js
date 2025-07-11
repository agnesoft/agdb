import {
  defineConfig,
  configDefaults,
  coverageConfigDefaults,
} from "vitest/config";

export default defineConfig({
  test: {
    environment: "jsdom",
    exclude: [...configDefaults.exclude, "e2e/*"],
    coverage: {
      provider: "v8",
      all: true,
      exclude: [
        ...coverageConfigDefaults.exclude,
        "e2e/*",
        "*.config.*",
        "./src/main.ts",
        "./src/App.vue",
        "src/tests/*",
        "**/types.ts",
      ],
      thresholds: {
        lines: 100,
        functions: 100,
        branches: 100,
        statements: 100,
      },
      reporter: ["text", "html", "json"],
    },

    globals: true,
  },
});
