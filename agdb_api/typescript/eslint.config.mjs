import eslint from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig(
    globalIgnores(["**/src/openapi.d.ts", "**/dist/", "examples/"]),
    {
        files: ["**/*.{js,ts}"],
        extends: [eslint.configs.recommended, tseslint.configs.recommended],
    },
);
