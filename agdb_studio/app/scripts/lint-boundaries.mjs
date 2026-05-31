import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const scriptPath = fileURLToPath(import.meta.url);
const studioRoot = path.resolve(path.dirname(scriptPath), "../..");
const eslintTarget = process.argv.length > 2 ? process.argv.slice(2) : ["."];

const result = spawnSync("pnpm exec eslint " + eslintTarget.join(" "), {
  cwd: studioRoot,
  stdio: "inherit",
  env: process.env,
  shell: true,
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
