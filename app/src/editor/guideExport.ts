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
