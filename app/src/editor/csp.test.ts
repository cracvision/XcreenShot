import { describe, expect, it } from "vitest";
import conf from "../../src-tauri/tauri.conf.json";

/** Sources a CSP directive allows. */
function directive(csp: string, name: string): string[] {
  const part = csp
    .split(";")
    .map((p) => p.trim())
    .find((p) => p.startsWith(`${name} `));
  return part ? part.split(/\s+/).slice(1) : [];
}

describe("content security policy of the installed app", () => {
  // The dev server does not apply the CSP, so a missing source only shows up in the
  // installed app: the editor canvas came up empty (build 4a, 2026-10-24).
  it("lets the editor load document pixels on Windows and macOS", () => {
    const img = directive(conf.app.security.csp, "img-src");
    expect(img).toContain("http://xsdoc.localhost");
    expect(img).toContain("xsdoc:");
  });

  it("lets the on-screen selection load frozen frames", () => {
    const connect = directive(conf.app.security.csp, "connect-src");
    expect(connect).toContain("http://xsframe.localhost");
    expect(connect).toContain("xsframe:");
  });
});
