// Main window in the Editor view (EDT-001). The Library view arrives in build 5.
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppShortcuts } from "../lib/appShortcuts";
import { openCapturePanel, openFiles } from "./actions";
import { Canvas } from "./Canvas";
import { Dialogs } from "./Dialogs";
import { useEditorKeys } from "./keys";
import { useMenuBar } from "./menus";
import { useAnnouncement } from "./announce";
import { SidePanel } from "./SidePanel";
import { useEditor } from "./state";
import { StatusBar } from "./StatusBar";
import { Strip } from "./Strip";
import { Toolbar } from "./Toolbar";

function Empty({ dropping }: { dropping: boolean }) {
  const { t } = useTranslation();
  return (
    <div className={`editor-empty${dropping ? " dropping" : ""}`}>
      <h1>{t("ui.editor.empty_title")}</h1>
      <p className="muted">{t("ui.editor.empty_help")}</p>
      <div className="row">
        <button type="button" className="btn primary" onClick={openCapturePanel}>
          {t("ui.editor.capture")}
        </button>
        <button type="button" className="btn" onClick={() => void openFiles()}>
          {t("ui.editor.open")}
        </button>
      </div>
    </div>
  );
}

export function Editor() {
  const { t } = useTranslation();
  useAppShortcuts();
  useEditorKeys();
  useMenuBar();
  const announcement = useAnnouncement();
  const ready = useEditor((s) => s.ready);
  const doc = useEditor((s) => s.doc);
  const toast = useEditor((s) => s.toast);
  const [dropping, setDropping] = useState(false);

  useEffect(() => {
    void useEditor.getState().boot();
  }, []);

  // Files dropped on the window open as documents (EDT-105). Floating images on the
  // canvas arrive with M2.
  useEffect(() => {
    let off: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent((e) => {
        if (e.payload.type === "enter" || e.payload.type === "over") setDropping(true);
        else if (e.payload.type === "leave") setDropping(false);
        else if (e.payload.type === "drop") {
          setDropping(false);
          void openFiles(e.payload.paths);
        }
      })
      .then((u) => (off = u));
    return () => off?.();
  }, []);

  if (!ready) return <div className="editor" />;

  return (
    <div className={`editor${dropping ? " dropping" : ""}`}>
      <Toolbar />
      <div className="editor-main">
        {doc ? <Canvas key={doc.documentId} doc={doc} /> : <Empty dropping={dropping} />}
        <SidePanel />
      </div>
      <StatusBar />
      <Strip />
      {toast && (
        <div className="editor-toast" role="status">
          {toast}
        </div>
      )}
      {dropping && doc && <div className="drop-hint">{t("ui.editor.drop_hint")}</div>}
      <Dialogs />
      <div className="sr-only" role="status" aria-live="polite">
        {announcement}
      </div>
    </div>
  );
}
