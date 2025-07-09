import { defineWorkspace } from "vitest/config";

export default defineWorkspace([
    "./agdb_api/typescript/vitest.config.ts",
    "./agdb_studio/**/vitest.config.mts",
    "./agdb_web/**/vitest.config.mts",
]);
