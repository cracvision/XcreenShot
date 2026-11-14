// Lists the third-party components that ship inside XcreenShot, with their licenses, for
// Help > About XcreenShot (APP-015). Only what ends up in the app: the Rust crates the
// workspace uses at run time on Windows and macOS, and the npm packages of the UI
// without the development tools. Run it after changing dependencies:
//
//   node scripts/licenses.mjs
//
// Writes app/src/help/licenses.json.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const run = (cmd, args, cwd) =>
  execFileSync(cmd, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
    shell: process.platform === "win32",
  });

const found = new Map();
const add = (name, version, license) => {
  const key = `${name}@${version}`;
  if (!found.has(key))
    found.set(key, { name, version, license: license || "see the package" });
};

// Rust: walk the resolved graph from the workspace members, normal dependencies only.
for (const target of ["x86_64-pc-windows-msvc", "aarch64-apple-darwin"]) {
  const meta = JSON.parse(
    run(
      "cargo",
      ["metadata", "--format-version", "1", "--filter-platform", target],
      root,
    ),
  );
  const packages = new Map(meta.packages.map((p) => [p.id, p]));
  const nodes = new Map(meta.resolve.nodes.map((n) => [n.id, n]));
  const members = new Set(meta.workspace_members);
  const seen = new Set();
  const stack = [...members];
  while (stack.length) {
    const id = stack.pop();
    if (seen.has(id)) continue;
    seen.add(id);
    const p = packages.get(id);
    if (p && !members.has(id)) add(p.name, p.version, p.license);
    for (const d of nodes.get(id)?.deps ?? []) {
      if (d.dep_kinds.some((k) => k.kind === null)) stack.push(d.pkg);
    }
  }
}
const rust = [...found.values()];
found.clear();

// JavaScript: the production tree of the UI.
const tree = JSON.parse(
  run(
    "npm",
    ["ls", "--omit=dev", "--all", "--json", "--long"],
    join(root, "app"),
  ),
);
const walk = (deps) => {
  for (const [name, d] of Object.entries(deps ?? {})) {
    if (d.version)
      add(
        name,
        d.version,
        typeof d.license === "string" ? d.license : d.license?.type,
      );
    walk(d.dependencies);
  }
};
walk(tree.dependencies);
const js = [...found.values()];

const byName = (a, b) =>
  a.name.localeCompare(b.name) || a.version.localeCompare(b.version);
const out = { rust: rust.sort(byName), js: js.sort(byName) };
writeFileSync(
  join(root, "app", "src", "help", "licenses.json"),
  JSON.stringify(out, null, 1) + "\n",
);
console.log(`${out.rust.length} crates, ${out.js.length} npm packages`);
