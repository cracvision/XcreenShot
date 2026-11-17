// Share > Local network… in the editor (SHR-044): the document shown, saved first.
import i18next from "i18next";
import { run } from "../lib/commands";
import { useEditor } from "../editor/state";
import { askSend } from "./lanState";

export async function sendActiveOnLan(): Promise<void> {
  const { active, doc } = useEditor.getState();
  if (!active || !doc) return;
  try {
    // What goes is the file: unsaved changes are saved first.
    if (doc.dirty || !doc.path) await run("document.save", { documentId: active });
    const path = useEditor.getState().doc?.path;
    if (!path) {
      useEditor.getState().showToast(i18next.t("ui.lan.save_first"));
      return;
    }
    await askSend([path]);
  } catch (e) {
    useEditor.getState().showError(e);
  }
}
