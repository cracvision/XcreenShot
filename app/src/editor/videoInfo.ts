// A video's capture details as rows of its information (VID-064, VID-054).

/** How a video was captured (VID-054), kept beside it. */
export interface CaptureDetails {
  capturedAt?: string;
  source?: string;
  app?: { name: string; version?: string } | null;
  apps?: string[];
  devices?: { microphone?: boolean; systemAudio?: boolean };
  encoder?: string;
  url?: string | null;
}

/** Its capture details as rows (VID-064): app, other apps, source and devices. */
export function captureRows(
  c: CaptureDetails | null | undefined,
  t: (key: string) => string,
): [string, string][] {
  if (!c) return [];
  const rows: [string, string][] = [];
  if (c.capturedAt)
    rows.push([t("ui.video.info.captured"), new Date(c.capturedAt).toLocaleString()]);
  if (c.app?.name)
    rows.push([t("ui.video.info.app"), [c.app.name, c.app.version].filter(Boolean).join(" ")]);
  const others = (c.apps ?? []).filter((a) => a !== c.app?.name);
  if (others.length) rows.push([t("ui.video.info.apps"), others.join(", ")]);
  if (c.url) rows.push([t("ui.video.info.url"), c.url]);
  if (c.source) {
    const known = ["region", "window", "screen", "slides", "system"];
    rows.push([
      t("ui.video.info.source"),
      known.includes(c.source) ? t(`ui.video.info.source_${c.source}`) : c.source,
    ]);
  }
  if (c.devices) {
    const on = [
      c.devices.microphone ? t("ui.video.info.microphone") : null,
      c.devices.systemAudio ? t("ui.video.info.system_audio") : null,
    ].filter((d): d is string => d !== null);
    rows.push([t("ui.video.info.devices"), on.length ? on.join(", ") : t("ui.video.info.no")]);
  }
  if (c.encoder) rows.push([t("ui.video.info.encoder"), c.encoder]);
  return rows;
}
