import { describe, expect, it } from "vitest";
import { captureRows } from "./videoInfo";

const t = (key: string) => key.replace("ui.video.info.", "");

describe("captureRows (VID-064)", () => {
  it("lists the app, the other apps, the source and the sound", () => {
    const rows = captureRows(
      {
        source: "window",
        app: { name: "Bloc de notas", version: "11.2" },
        apps: ["Bloc de notas", "Edge"],
        devices: { microphone: true, systemAudio: false },
        encoder: "hardware",
      },
      t,
    );
    expect(rows).toEqual([
      ["app", "Bloc de notas 11.2"],
      ["apps", "Edge"],
      ["source", "source_window"],
      ["devices", "microphone"],
      ["encoder", "hardware"],
    ]);
  });

  it("says nothing for a video from elsewhere, and no sound when none", () => {
    expect(captureRows(null, t)).toEqual([]);
    expect(captureRows({ source: "camera", devices: {} }, t)).toEqual([
      ["source", "camera"],
      ["devices", "no"],
    ]);
  });
});
