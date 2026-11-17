// Builds the portable .zip of Windows (APP-038) from the release build that
// `npm run package` just made: the app, the `xcreenshot` CLI, AGENTS.md and
// `portable.flag`, which makes XcreenShot keep its data and library in the folder.
// Usage (after `npm run package` in app/): node scripts/make-portable.mjs
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

if (process.platform !== "win32") {
  console.log("The portable .zip is only made on Windows.");
  process.exit(0);
}
const root = fileURLToPath(new URL("..", import.meta.url));
const version = JSON.parse(
  readFileSync(join(root, "app", "package.json"), "utf8"),
).version;
const release = join(root, "target", "release");
const out = join(release, "bundle", "portable");
const folder = join(out, "XcreenShot");
rmSync(out, { recursive: true, force: true });
mkdirSync(folder, { recursive: true });
// The app keeps its name: on Windows "XcreenShot.exe" and the CLI's "xcreenshot.exe"
// would be the same file.
copyFileSync(
  join(release, "xcreenshot-app.exe"),
  join(folder, "xcreenshot-app.exe"),
);
copyFileSync(join(release, "xcreenshot.exe"), join(folder, "xcreenshot.exe"));
copyFileSync(join(root, "AGENTS.md"), join(folder, "AGENTS.md"));
writeFileSync(
  join(folder, "portable.flag"),
  "XcreenShot portable: settings, data and the library stay in this folder (APP-038).\r\n",
);
const zip = join(out, `XcreenShot_${version}_x64-portable.zip`);
execFileSync(
  "powershell",
  [
    "-NoProfile",
    "-Command",
    `Compress-Archive -Path '${folder}' -DestinationPath '${zip}' -Force`,
  ],
  { stdio: "inherit" },
);
console.log(`Portable: ${zip}`);
