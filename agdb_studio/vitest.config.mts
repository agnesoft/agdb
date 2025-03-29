import { mergeConfig } from "vitest/config";
import viteConfig from "./vite.config.mjs";
import vitestShared from "@agdb-studio/config/vitest.shared";

export default mergeConfig(viteConfig, vitestShared);
