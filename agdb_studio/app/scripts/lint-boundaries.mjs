import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const scriptPath = fileURLToPath(import.meta.url);
const studioRoot = path.resolve(path.dirname(scriptPath), "../..");
const rawArgs = process.argv.slice(2);

const forwardedArgs = [];
for (let index = 0; index < rawArgs.length; index += 1) {
  const arg = rawArgs[index];

  if (arg === "--filter") {
    index += 1;
    continue;
  }

  if (arg.startsWith("--filter=")) {
    continue;
  }

  forwardedArgs.push(arg);
}

const eslintTarget =
  forwardedArgs.length > 0 ? forwardedArgs : ["--max-warnings=0", "."];
const packageManagerExecPath = process.env.npm_execpath;

const command = packageManagerExecPath
  ? process.execPath
  : process.platform === "win32"
    ? "pnpm.cmd"
    : "pnpm";

const args = packageManagerExecPath
  ? [packageManagerExecPath, "exec", "eslint", ...eslintTarget]
  : ["exec", "eslint", ...eslintTarget];

const result = spawnSync(command, args, {
  cwd: studioRoot,
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
