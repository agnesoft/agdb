import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
    eslint.configs.recommended,
    tseslint.configs.recommended,
    {
        ignores: ["**/src/openapi.d.ts", "**/.gitignore", "**/dist/"],
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            // globals: globals.browser,
            parserOptions: {
                parser: typescriptEslint.parser,
                projectService: true,
                tsconfigRootDir: import.meta.dirname,
            },
        },
    },
);
