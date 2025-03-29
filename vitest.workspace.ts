import { defineWorkspace } from "vitest/config";

export default defineWorkspace([
    "./agdb_api/typescript/vitest.config.ts",
    "./agdb_studio/**/vite.config.mts",
    "./agdb_web/**/vite.config.mts",
]);
