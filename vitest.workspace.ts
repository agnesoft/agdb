import { defineWorkspace } from 'vitest/config'

export default defineWorkspace([
  "./agdb_web/vitest.config.ts",
  "./agdb_studio/vitest.config.mts",
  "./agdb_studio/libs/notification/vite.config.mts",
  "./agdb_studio/libs/design/vite.config.mts",
  "./agdb_api/typescript/vitest.config.ts"
])
