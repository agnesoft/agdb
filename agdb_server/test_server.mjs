// Runs the given command against a freshly built agdb_server listening on http://localhost:3000.
// Usage: node test_server.mjs <command> [args...]
import { spawn, spawnSync } from "node:child_process";
import { rmSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ADDRESS = "http://localhost:3000";
const STARTUP_TIMEOUT_MS = 60_000;
const SHUTDOWN_TIMEOUT_MS = 10_000;
const root = fileURLToPath(new URL("..", import.meta.url));
const target = process.env.CARGO_TARGET_DIR ?? join(root, "target");
const binary = join(target, "release", `agdb_server${process.platform === "win32" ? ".exe" : ""}`);
const [command, ...args] = process.argv.slice(2);

function clean() {
    rmSync("agdb_server.yaml", { force: true });
    rmSync("agdb_server_data", { recursive: true, force: true });
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForServer(server) {
    const deadline = Date.now() + STARTUP_TIMEOUT_MS;

    while (Date.now() < deadline) {
        if (server.exitCode !== null) {
            throw new Error(`agdb_server exited with ${server.exitCode}`);
        }

        try {
            if ((await fetch(`${ADDRESS}/api/v1/status`)).ok) {
                return;
            }
        } catch {
            // not listening yet
        }

        await sleep(200);
    }

    throw new Error(`agdb_server did not start within ${STARTUP_TIMEOUT_MS} ms`);
}

async function shutdown(server) {
    const exited = new Promise((resolve) => server.once("exit", resolve));

    try {
        const login = await fetch(`${ADDRESS}/api/v1/user/login`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ username: "admin", password: "admin" }),
        });
        const token = await login.json();
        await fetch(`${ADDRESS}/api/v1/admin/shutdown`, {
            method: "POST",
            headers: { Authorization: `Bearer ${token}` },
        });
    } catch (error) {
        console.error(`Graceful agdb_server shutdown failed: ${error}`);
    }

    if (server.exitCode === null && (await Promise.race([exited.then(() => true), sleep(SHUTDOWN_TIMEOUT_MS)])) !== true) {
        server.kill();
        await exited;
    }
}

if (!command) {
    console.error("Usage: node test_server.mjs <command> [args...]");
    process.exit(2);
}

clean();

const build = spawnSync("cargo", ["build", "-r", "-p", "agdb_server"], { stdio: "inherit", shell: process.platform === "win32" });

if (build.status !== 0) {
    process.exit(build.status ?? 1);
}

const server = spawn(binary, { stdio: "inherit" });
let status = 1;

try {
    await waitForServer(server);
    status = spawnSync(command, args, { stdio: "inherit", shell: process.platform === "win32" }).status ?? 1;
} catch (error) {
    console.error(error.message);
} finally {
    await shutdown(server);
    clean();
}

process.exit(status);
