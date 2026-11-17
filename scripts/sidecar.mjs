// Builds the `xcreenshot` CLI and puts it where the installer takes it from, so it is
// installed next to the app (APP-028, APP-029: MCP clients start `xcreenshot mcp`).
// Usage: node scripts/sidecar.mjs [--debug]
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const debug = process.argv.includes("--debug");
const profile = debug ? "debug" : "release";
const run = (cmd, args) =>
  execFileSync(cmd, args, {
    cwd: root,
    stdio: ["ignore", "pipe", "inherit"],
  }).toString();

run("cargo", ["build", "-p", "xs-cli", ...(debug ? [] : ["--release"])]);
const triple = /host: (\S+)/.exec(run("rustc", ["-vV"]))[1];
const ext = process.platform === "win32" ? ".exe" : "";
const dir = root + "app/src-tauri/binaries/";
mkdirSync(dir, { recursive: true });
copyFileSync(
  `${root}target/${profile}/xcreenshot${ext}`,
  `${dir}xcreenshot-${triple}${ext}`,
);
console.log(`CLI ready for the installer: binaries/xcreenshot-${triple}${ext}`);
