// What the template entries do (EDT-095 to EDT-101): commands of the core, with the
// editor's notices and errors. Each change is one undo step.
import i18next from "i18next";
import { open, save } from "@tauri-apps/plugin-dialog";
import { run } from "../lib/commands";
import { showView, useMainView } from "../library/state";
import { useEditor, usePanel } from "./state";
import {
  openPicker,
  useSaveTemplate,
  useZones,
  zoneAt,
  type ImageRef,
  type TemplateSettings,
} from "./templateUi";

const IMAGE_FILES = ["png", "jpg", "jpeg", "xshot"];

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** A new document from a template with these images in its zones (EDT-095); the
 * Template tab opens. */
export async function newFromTemplate(template: string, images: ImageRef[]): Promise<boolean> {
  try {
    const r = await run<{ warnings: string[] }>("document.from_template", { template, images });
    const warning = r.warnings[0];
    if (warning) useEditor.getState().showToast(warning);
    showView("editor");
    usePanel.getState().set({ visible: true, tab: "template" });
    return true;
  } catch (e) {
    fail(e);
    return false;
  }
}

/** File > New from template (EDT-095): with the Library's selected images, Combine. */
export function pickTemplate(images: ImageRef[] = []): void {
  openPicker(images);
}

/** File > Apply template (EDT-095): the document shown is the first image. */
export function applyTemplate(): void {
  const id = useEditor.getState().active;
  if (id) openPicker([{ documentId: id }]);
}

/** A change of the shown template (EDT-097 to EDT-099, EDT-101). */
export async function templateCommand(
  action: string,
  params: Record<string, unknown> = {},
): Promise<void> {
  const documentId = useEditor.getState().active;
  if (!documentId) return;
  try {
    await run("document.template", { documentId, action, ...params });
  } catch (e) {
    fail(e);
  }
}

export function setTemplate(settings: Partial<TemplateSettings>): Promise<void> {
  return templateCommand("settings", { settings });
}

/** A file (dropped or chosen) into a zone (EDT-098). */
export function fillZone(zone: string, path: string): Promise<void> {
  return templateCommand("fill_zone", { zone, path });
}

/** An image file dropped on a template's zone fills it (EDT-098). `at` is where it
 * was dropped, in the window's physical pixels. */
export function dropOnZone(paths: string[], at: { x: number; y: number }): boolean {
  const { doc } = useEditor.getState();
  const pointAt = useZones.getState().pointAt;
  const file = paths.find((p) => /\.(png|jpe?g|xshot)$/i.test(p));
  if (!doc || !pointAt || !file || useMainView.getState().view !== "editor") return false;
  const dpr = window.devicePixelRatio || 1;
  const p = pointAt(at.x / dpr, at.y / dpr);
  const zone = p ? zoneAt(doc.objects, p) : null;
  if (!zone) return false;
  void fillZone(zone.id, file);
  return true;
}

/** Replace image… (EDT-098). */
export async function replaceZoneImage(zone: string): Promise<void> {
  const chosen = await open({
    multiple: false,
    filters: [{ name: i18next.t("ui.editor.open_filter"), extensions: IMAGE_FILES }],
  });
  if (typeof chosen === "string") await fillZone(zone, chosen);
}

/** The Capture button of an empty zone (EDT-098): a region capture goes into it. */
export async function captureIntoZone(zone: string): Promise<void> {
  const documentId = useEditor.getState().active;
  if (!documentId) return;
  try {
    const r = await run<{ status: string; path?: string }>("capture.region", {
      open: "none",
      copy: false,
    });
    if (r.status === "completed" && r.path)
      await run("document.template", { documentId, action: "fill_zone", zone, path: r.path });
  } catch (e) {
    fail(e);
  }
}

/** File > Save as template (EDT-101). */
export function askSaveTemplate(): void {
  if (useEditor.getState().active) useSaveTemplate.setState({ open: true });
}

export async function saveAsTemplate(
  name: string,
  family: string,
  emptyZones: boolean,
  replace = false,
): Promise<boolean> {
  const documentId = useEditor.getState().active;
  if (!documentId) return false;
  try {
    await run("document.save_as_template", { documentId, name, family, emptyZones, replace });
    useEditor.getState().showToast(i18next.t("ui.editor.template.saved", { name }));
    return true;
  } catch (e) {
    fail(e);
    return false;
  }
}

/** File > Import > Template (EDT-101), or a .xtemplate dropped or opened: it joins My
 * templates and the picker opens. */
export async function importTemplate(path?: string): Promise<void> {
  try {
    const chosen =
      path ??
      ((await open({
        filters: [{ name: i18next.t("ui.editor.template.file"), extensions: ["xtemplate"] }],
      })) as string | null);
    if (!chosen) return;
    await run("editor.templates", { action: "import", path: chosen });
    openPicker([]);
  } catch (e) {
    fail(e);
  }
}

/** Export… of an own template (EDT-101). */
export async function exportTemplate(id: string, name: string): Promise<void> {
  try {
    const path = await save({
      defaultPath: `${name}.xtemplate`,
      filters: [{ name: i18next.t("ui.editor.template.file"), extensions: ["xtemplate"] }],
    });
    if (path) await run("editor.templates", { action: "export", id, path });
  } catch (e) {
    fail(e);
  }
}

/** Apply theme (EDT-100): the selected objects, or all. */
export async function applyThemeToObjects(themeId: string): Promise<void> {
  const { active, selection } = useEditor.getState();
  if (!active) return;
  try {
    await run("document.apply_theme", {
      documentId: active,
      themeId,
      ...(selection.length > 0 ? { objectIds: selection } : {}),
    });
  } catch (e) {
    fail(e);
  }
}
