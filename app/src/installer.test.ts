import { describe, expect, it } from "vitest";
import bundle from "../src-tauri/tauri.bundle.json";
import hooks from "../src-tauri/installer-hooks.nsh?raw";

describe("Windows installer (APP-028)", () => {
  // An MCP client keeps the CLI running, and overwriting it stopped the install with
  // "Error opening file for writing" (2026-11-05).
  it("sets a running CLI aside instead of failing on it", () => {
    expect(bundle.bundle.windows.nsis.installerHooks).toBe("installer-hooks.nsh");
    expect(hooks).toContain("!macro NSIS_HOOK_PREINSTALL");
    expect(hooks).toContain('Rename "$INSTDIR\\xcreenshot.exe"');
  });
});
