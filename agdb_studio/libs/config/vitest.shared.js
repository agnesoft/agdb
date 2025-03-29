import {
  defineConfig,
  configDefaults,
  coverageConfigDefaults,
} from "vitest/config";
import path from "path";

export default defineConfig({
  test: {
    environment: "jsdom",
    exclude: [...configDefaults.exclude, "e2e/*"],
    // root: path.resolve(__dirname, "."),
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
