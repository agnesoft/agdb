import eslint from "@eslint/js";
import typescriptEslint from "typescript-eslint";

export default typescriptEslint.config({
  extends: [
    eslint.configs.recommended,
    ...typescriptEslint.configs.recommended,
  ],
  languageOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    // globals: globals.browser,
    parserOptions: {
      parser: typescriptEslint.parser,
    },
  },
  files: ["**/*.ts"],
});
