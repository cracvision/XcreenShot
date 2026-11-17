import { beforeEach, describe, expect, it } from "vitest";
import { actionOff, captureModes, featureOn } from "./features";
import type { SettingEntry } from "./lib/commands";
import { useApp } from "./store";

const entry = (key: string, value: unknown) => ({ key, value }) as unknown as SettingEntry;

beforeEach(() => {
  useApp.setState({ settings: [] });
});

describe("smart features on and off (SMT-002)", () => {
  it("are on unless switched off, also before the settings arrive", () => {
    expect(featureOn([], "autoRedact")).toBe(true);
    expect(featureOn([entry("smart.autoRedact.enabled", false)], "autoRedact")).toBe(false);
  });

  it("an editor action of a feature off does nothing", () => {
    expect(actionOff("auto_redact")).toBe(false);
    useApp.setState({ settings: [entry("smart.autoRedact.enabled", false)] });
    expect(actionOff("auto_redact")).toBe(true);
    expect(actionOff("extract_text")).toBe(false);
    expect(actionOff("crop")).toBe(false);
  });

  it("the capture modes of a feature off leave the list, unless chosen", () => {
    const modes = ["region", "text", "steps"] as const;
    const off = [entry("smart.steps.enabled", false), entry("smart.extractText.enabled", false)];
    expect(captureModes(modes, off, "region")).toEqual(["region"]);
    expect(captureModes(modes, off, "steps")).toEqual(["region", "steps"]);
    expect(captureModes(modes, [], "region")).toEqual(["region", "text", "steps"]);
  });
});
