import eslint from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginVue from "eslint-plugin-vue";
import typescriptEslint from "typescript-eslint";

export default typescriptEslint.config(
    { ignores: ["*.d.ts", "**/coverage", "**/dist", "**/.gitignore"] },
    {
        extends: [
            eslint.configs.recommended,
            ...typescriptEslint.configs.recommended,
            ...eslintPluginVue.configs["flat/recommended"],
        ],
        files: ["**/*.{ts,vue}"],
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            parserOptions: {
                parser: typescriptEslint.parser,
            },
        },
    },
    eslintConfigPrettier,
    {
        files: ["src/composables/graph/prototype/**/*.{ts,vue}"],
        rules: { "@typescript-eslint/no-explicit-any": "off" },
    },
);
