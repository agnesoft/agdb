import eslint from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
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
  path.join(workspaceRoot, "libs/*/*/tsconfig.app.json"),
];

export default defineConfig(
  globalIgnores(["*.d.ts", "**/coverage", "**/dist"]),
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
      parserOptions: {
        parser: typescriptEslint.parser,
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: [".vue"],
      },
    },

    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "import/named": "off",
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
  // Disable no-undef for Vue SFCs — TypeScript handles undefined-variable
  // checking, and defineConfig's files intersection means .vue files don't
  // inherit the tseslint override that disables it for .ts files.
  {
    files: ["**/*.vue"],
    rules: { "no-undef": "off" },
  },
  eslintConfigPrettier,
  {
    files: ["**/graph/prototype/**/*.{ts,vue}"],
    rules: { "@typescript-eslint/no-explicit-any": "off" },
  },
  ...boundaryConfig,
);
