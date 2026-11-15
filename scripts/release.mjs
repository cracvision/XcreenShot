// A release for the updater (APP-036): the signed installer, the portable .zip and the
// manifest of the Stable channel. With --publish it uploads them with `gh` to the public
// releases repository (decided by Carlos on 2026-11-15); never without being asked.
//
//   node scripts/release.mjs                 build, sign and write target/release/updates/
//   node scripts/release.mjs --publish       …and upload (release v<version> + manifest)
//   node scripts/release.mjs --promote       Deferred channel := what Stable has now
//
// Needs the private key in ~/.xcreenshot-release/updater.key (outside the repository;
// without it there are no updates: keep a copy in a safe place) and the notes of the
// version in release-notes/<version>.json: {"en": "…", "es": "…"}.
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const REPO = "cracvision/xcreenshot-releases";
const root = fileURLToPath(new URL("..", import.meta.url));
const app = join(root, "app");
const version = JSON.parse(
  readFileSync(join(app, "package.json"), "utf8"),
).version;
const out = join(root, "target", "release", "updates");
const args = new Set(process.argv.slice(2));
// Only npx needs a shell on Windows; through one, arguments with spaces (a title,
// the notes) would be split.
const run = (cmd, a, opts = {}) =>
  execFileSync(cmd, a, {
    stdio: "inherit",
    shell: process.platform === "win32" && cmd === "npx",
    ...opts,
  });

if (args.has("--promote")) {
  // Deferred gets what Stable has, after its 30 days without a fix (APP-036): a
  // decision made by hand when the time comes.
  run("gh", [
    "release",
    "download",
    "updates",
    "-R",
    REPO,
    "-p",
    "stable.json",
    "-D",
    out,
    "--clobber",
  ]);
  copyFileSync(join(out, "stable.json"), join(out, "deferred.json"));
  run("gh", [
    "release",
    "upload",
    "updates",
    join(out, "deferred.json"),
    "-R",
    REPO,
    "--clobber",
  ]);
  process.exit(0);
}

const keyFile = join(homedir(), ".xcreenshot-release", "updater.key");
if (!existsSync(keyFile)) {
  console.error(`No signing key at ${keyFile}: updates cannot be signed.`);
  process.exit(1);
}
const notesFile = join(root, "release-notes", `${version}.json`);
if (!existsSync(notesFile)) {
  console.error(
    `Write the notes of ${version} first: ${notesFile} with {"en": "…", "es": "…"}.`,
  );
  process.exit(1);
}
const notes = JSON.parse(readFileSync(notesFile, "utf8"));

// The build, signed, with the updater's files (.sig next to the installer).
run("node", [join(root, "scripts", "sidecar.mjs")]);
run(
  "npx",
  [
    "tauri",
    "build",
    "--config",
    "src-tauri/tauri.bundle.json",
    "--config",
    "src-tauri/tauri.release.json",
  ],
  {
    cwd: app,
    env: {
      ...process.env,
      TAURI_SIGNING_PRIVATE_KEY: readFileSync(keyFile, "utf8"),
      TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "",
    },
  },
);
run("node", [join(root, "scripts", "make-portable.mjs")]);

const nsis = join(root, "target", "release", "bundle", "nsis");
const setup = readdirSync(nsis).find(
  (f) => f.startsWith(`XcreenShot_${version}_`) && f.endsWith("-setup.exe"),
);
if (!setup || !existsSync(join(nsis, `${setup}.sig`))) {
  console.error("The signed installer is missing.");
  process.exit(1);
}
mkdirSync(out, { recursive: true });
const tag = `v${version}`;
const manifest = {
  version,
  notes: JSON.stringify(notes),
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: readFileSync(join(nsis, `${setup}.sig`), "utf8"),
      url: `https://github.com/${REPO}/releases/download/${tag}/${setup}`,
    },
  },
};
writeFileSync(
  join(out, "stable.json"),
  JSON.stringify(manifest, null, 2) + "\n",
);
console.log(`Manifest: ${join(out, "stable.json")}`);

if (args.has("--publish")) {
  const portable = join(
    root,
    "target",
    "release",
    "bundle",
    "portable",
    `XcreenShot_${version}_x64-portable.zip`,
  );
  run("gh", [
    "release",
    "create",
    tag,
    join(nsis, setup),
    portable,
    "-R",
    REPO,
    "--title",
    `XcreenShot ${version}`,
    "--notes",
    `${notes.es ?? ""}\n\n${notes.en ?? ""}`,
  ]);
  run("gh", [
    "release",
    "upload",
    "updates",
    join(out, "stable.json"),
    "-R",
    REPO,
    "--clobber",
  ]);
}
