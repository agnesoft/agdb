import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { getVueComponentEntries } from "./componentEntries";

const componentEntries = getVueComponentEntries(path.resolve(__dirname, "src"));

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  base: "/studio/",

  // not really needed right now, but might be useful in the future
  build: {
    lib: {
      entry: componentEntries,
      formats: ["es"],
    },
    rollupOptions: {
      input: componentEntries,
      external: ["vue"],
      output: {
        preserveModules: true,
        preserveModulesRoot: "src",
        globals: {
          vue: "Vue",
        },
        entryFileNames: "[name].js",
      },
    },
  },
});
