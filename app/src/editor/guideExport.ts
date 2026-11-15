// Exporting a guide of steps (SMT-030, SMT-032 to SMT-034): the format, where it goes
// (a file or a folder, chosen in the system's dialog) and one command of the core.
import { open, save } from "@tauri-apps/plugin-dialog";
import i18next from "i18next";
import { create } from "zustand";
import { isCommandError, run } from "../lib/commands";
import { useEditor } from "./state";
import { plainText, templateOf } from "./templateUi";

export const GUIDE_FORMATS = [
  "docx",
  "pptx",
  "pdf",
  "md",
  "html",
  "html1",
  "images",
  "clipboard",
] as const;
export type GuideFormat = (typeof GUIDE_FORMATS)[number];

/** Files of one format get a file dialog; Markdown, HTML and pictures, a folder. */
const FILE_EXT: Partial<Record<GuideFormat, string>> = { docx: "docx", pptx: "pptx", pdf: "pdf" };

function title(): string {
  const doc = useEditor.getState().doc;
  const o = doc?.objects.find((x) => (x as { slot?: string }).slot === "title");
  const text = o && o.type === "text" ? plainText(o.text, 120) : "";
  return (text || doc?.name || "Guide").replace(/[\\/:*?"<>|]/g, " ").trim();
}

/** Exports the guide shown in `format`, asking where. */
export async function exportGuide(format: GuideFormat): Promise<void> {
  const doc = useEditor.getState().doc;
  if (!doc || templateOf(doc)?.id !== "guide") return;
  await exportTo({ documentId: doc.documentId }, title(), format);
}

/** Exports a guide's file of the library without opening it (SMT-035). */
export async function exportGuideFile(
  path: string,
  name: string,
  format: GuideFormat,
): Promise<void> {
  await exportTo({ path }, name, format);
}

async function exportTo(
  target: { documentId: string } | { path: string },
  name: string,
  format: GuideFormat,
): Promise<void> {
  const ed = useEditor.getState();
  const t = i18next.t.bind(i18next);
  const params: Record<string, unknown> = { ...target, format };
  try {
    if (format === "clipboard") {
      await run("smart.guide_export", params);
      ed.showToast(t("ui.guide.copied"));
      return;
    }
    // Pictures one by one: their options first (SMT-034).
    if (format === "images") {
      useGuideImages.setState({ ask: { target, name } });
      return;
    }
    const ext = FILE_EXT[format];
    if (ext) {
      const path = await save({
        defaultPath: `${name}.${ext}`,
        filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
      });
      if (!path) return;
      params.out = path;
    } else {
      const dir = await open({ directory: true, title: t(`ui.guide.export.${format}`) });
      if (typeof dir !== "string") return;
      params.out = dir;
      if (format === "html1") {
        params.format = "html";
        params.singleFile = true;
      }
    }
    // A folder or page of that name: the user chooses (SMT-032).
    if (format === "md" || format === "html" || format === "html1") params.onConflict = "ask";
    await finishExport(params);
  } catch (e) {
    ed.showError(e);
  }
}

/** The options of a guide's pictures one by one (SMT-034). */
export interface GuideImagesOptions {
  imageFormat: "png" | "jpg";
  /** 25 to 400 %. */
  scale: number;
  /** A saved effect's id, or none. */
  effects: string;
  /** The names: `<base>_01.png`… from `start`. */
  base: string;
  start: number;
  /** Also `pasos.txt` with each step's text. */
  withText: boolean;
}

/** A guide waiting for the options of its pictures one by one (SMT-034). */
export const useGuideImages = create<{
  ask: { target: { documentId: string } | { path: string }; name: string } | null;
}>(() => ({ ask: null }));

/** Exports the pictures with `o`, asking for the folder. */
export async function exportGuideImages(o: GuideImagesOptions): Promise<void> {
  const ask = useGuideImages.getState().ask;
  if (!ask) return;
  const ed = useEditor.getState();
  const t = i18next.t.bind(i18next);
  try {
    const dir = await open({ directory: true, title: t("ui.guide.export.images") });
    if (typeof dir !== "string") return;
    useGuideImages.setState({ ask: null });
    await finishExport({
      ...ask.target,
      format: "images",
      out: dir,
      imageFormat: o.imageFormat,
      scale: o.scale,
      withText: o.withText,
      ...(o.effects ? { effects: o.effects } : {}),
      naming: { base: o.base.trim() || ask.name, start: o.start },
    });
  } catch (e) {
    ed.showError(e);
  }
}

/** Runs the export; a name already taken asks Replace or Create another first. */
async function finishExport(params: Record<string, unknown>): Promise<void> {
  const ed = useEditor.getState();
  const t = i18next.t.bind(i18next);
  try {
    const r = await run<{ paths: string[] }>("smart.guide_export", params);
    ed.showToast(t("ui.guide.exported", { path: r.paths[0] ?? "" }));
  } catch (e) {
    const conflict = isCommandError(e)
      ? (e.details as { conflict?: string } | undefined)?.conflict
      : undefined;
    if (!conflict) throw e;
    useGuideConflict.setState({
      ask: {
        name: conflict,
        choose: (choice) => {
          useGuideConflict.setState({ ask: null });
          if (choice)
            void finishExport({ ...params, onConflict: choice }).catch((err: unknown) =>
              ed.showError(err),
            );
        },
      },
    });
  }
}

/** A guide export waiting for Replace or Create another (SMT-032). */
export const useGuideConflict = create<{
  ask: { name: string; choose: (choice: "replace" | "new" | null) => void } | null;
}>(() => ({ ask: null }));
