// Main window (EDT-001): the Editor view and the Library view (LIB-001), switched at
// the top; the menu bar, dialogs and notices are shared by both.
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import logoDark from "../assets/brand/logo-dark.webp";
import logoLight from "../assets/brand/logo-light.webp";
import { Keys } from "../components/Keys";
import { MenuBar } from "./MenuBar";
import { ContextMenu } from "../components/Menu";
import { displayKeys } from "../lib/accelerator";
import { useAppShortcuts } from "../lib/appShortcuts";
import { useApp } from "../store";
import { openCapturePanel, openFiles } from "./actions";
import { Canvas } from "./Canvas";
import { Dialogs } from "./Dialogs";
import { useEditorKeys } from "./keys";
import { closeContextMenu, useContextMenu, useMenuBar } from "./menus";
import { useAnnouncement } from "./announce";
import { SidePanel } from "./SidePanel";
import { useEditor } from "./state";
import { StatusBar } from "./StatusBar";
import { Strip } from "./Strip";
import { Toolbar } from "./Toolbar";
import { LibraryView } from "../library/LibraryView";
import { useMainView } from "../library/state";
import { UndoNotice } from "./UndoNotice";

function Empty({ dropping }: { dropping: boolean }) {
  const { t } = useTranslation();
  // Plain values: a selector that builds an array would re-render without end.
  const shortcut = useApp(
    (s) => s.shortcuts?.rows.find((r) => r.command === "capture.capture")?.shortcut,
  );
  const platform = useApp((s) => s.shortcuts?.platform ?? "windows");
  const layout = useApp((s) => s.layout);
  const keys = shortcut ? displayKeys(shortcut, platform, layout) : null;
  return (
    <div className={`editor-empty${dropping ? " dropping" : ""}`}>
      <div className="empty-card">
        {/* The logo stays while no capture is shown; its REC dot blinks as if
            recording (Carlos, 2026-11-05). */}
        <div className="brand-logo">
          <img className="logo-light" src={logoLight} alt="XcreenShot" draggable={false} />
          <img className="logo-dark" src={logoDark} alt="XcreenShot" draggable={false} />
          <span className="rec-dot" aria-hidden="true" />
        </div>
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
        {keys && (
          <p className="empty-keys muted">
            {t("ui.editor.empty_shortcut")} <Keys keys={keys} />
          </p>
        )}
      </div>
    </div>
  );
}

/** macOS keeps the system menu bar; Windows gets the drawn one (EDT-002). */
const IS_MAC = navigator.userAgent.includes("Mac");

/** The right-click menu of the canvas or the toolbar, when one is open. */
function EditorContextMenu() {
  const menu = useContextMenu();
  return (
    <ContextMenu
      open={menu.open}
      x={menu.x}
      y={menu.y}
      label={menu.label}
      items={menu.items}
      onClose={closeContextMenu}
    />
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
  const view = useMainView((s) => s.view);
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
      {!IS_MAC && <MenuBar />}
      {view === "library" ? (
        <LibraryView />
      ) : (
        <>
          <Toolbar />
          <div className="editor-main">
            {doc ? <Canvas key={doc.documentId} doc={doc} /> : <Empty dropping={dropping} />}
            <SidePanel />
          </div>
          <StatusBar />
          {/* The recent tray belongs to the Editor view (LIB-001). */}
          <Strip />
        </>
      )}
      <UndoNotice />
      {toast && (
        <div className="editor-toast" role="status" key={toast}>
          {toast}
        </div>
      )}
      {dropping && doc && <div className="drop-hint">{t("ui.editor.drop_hint")}</div>}
      <Dialogs />
      <EditorContextMenu />
      <div className="sr-only" role="status" aria-live="polite">
        {announcement}
      </div>
    </div>
  );
}
