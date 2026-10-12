// APP-054: fails if any Rust or npm dependency is an analytics or crash-reporting
// component. Usage: node scripts/check-no-telemetry.mjs
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const DENY = [
  "sentry", "bugsnag", "crashpad", "breakpad", "rollbar", "raygun", "appcenter",
  "posthog", "mixpanel", "segment", "amplitude", "analytics", "telemetry", "datadog",
  "newrelic", "honeycomb", "opentelemetry", "aptabase", "google-analytics", "firebase",
];

const cargo = [...readFileSync(root + "Cargo.lock", "utf8").matchAll(/^name = "([^"]+)"/gm)].map((m) => m[1]);
const npmLock = JSON.parse(readFileSync(root + "app/package-lock.json", "utf8"));
const npm = Object.keys(npmLock.packages ?? {}).map((p) => p.replace(/^.*node_modules\//, "")).filter(Boolean);

const hits = [...cargo.map((n) => `cargo:${n}`), ...npm.map((n) => `npm:${n}`)].filter((n) =>
  DENY.some((d) => new RegExp(`(^|[:/@_-])${d}($|[_-])`).test(n.toLowerCase())),
);
if (hits.length) {
  console.error("Telemetry/analytics dependencies found:\n  " + hits.join("\n  "));
  process.exit(1);
}
console.log(`No telemetry dependencies (${cargo.length} crates, ${npm.length} npm packages checked).`);
