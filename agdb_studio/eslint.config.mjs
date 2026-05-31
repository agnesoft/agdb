import eslint from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginVue from "eslint-plugin-vue";
import typescriptEslint from "typescript-eslint";
import importPlugin from "eslint-plugin-import";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { boundaryConfig } from "./eslint.boundaries.mjs";

const workspaceRoot = path.dirname(fileURLToPath(import.meta.url));

const typescriptProjects = [
  path.join(workspaceRoot, "app/tsconfig.app.json"),
  path.join(workspaceRoot, "app/tsconfig.vitest.json"),
  path.join(workspaceRoot, "libs/*/*/tsconfig.app.json"),
  path.join(workspaceRoot, "libs/*/*/tsconfig.vitest.json"),
  path.join(workspaceRoot, "libs/*/*/tsconfig.node.json"),
];

export default typescriptEslint.config(
  { ignores: ["*.d.ts", "**/coverage", "**/dist", "**/.gitignore"] },
  {
    extends: [
      eslint.configs.recommended,
      importPlugin.flatConfigs.recommended,
      ...typescriptEslint.configs.recommended,
      ...eslintPluginVue.configs["flat/recommended"],
    ],
    files: ["**/*.{ts,vue,spec.ts}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      // globals: globals.browser,
      parserOptions: {
        parser: typescriptEslint.parser,
      },
    },

    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "import/named": "off",
      // "import/extensions": "off",
      // "import/no-unresolved": ["error", { commonjs: true, amd: true }],
      // "import/named": "error",
      // "import/namespace": "error",
      // "import/default": "error",
      // "import/export": "error",
    },
    settings: {
      "import/resolver": {
        node: {
          extensions: [".js", ".mjs", ".ts", ".d.ts", ".vue", ".json"],
        },
        typescript: {
          alwaysTryTypes: true,
          noWarnOnMultipleProjects: true,
          project: typescriptProjects,
        },
      },
    },
  },
  eslintConfigPrettier,
  {
    files: ["**/graph/prototype/**/*.{ts,vue}"],
    rules: { "@typescript-eslint/no-explicit-any": "off" },
  },
  ...boundaryConfig,
);
