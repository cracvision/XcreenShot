// Main window (EDT-001): the Editor view and the Library view (LIB-001), switched at
// the top; the menu bar, dialogs and notices are shared by both.
import { RedactBatch } from "./RedactBatch";
import { ShortcutSheet } from "./ShortcutSheet";
import { MoveComputer } from "../library/MoveComputer";
import { TagDialogs } from "../library/TagDialogs";
import { askImport } from "../library/importState";
import { QuickView } from "../library/QuickView";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { run } from "../lib/commands";
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
import { PendingShare } from "./PendingShare";
import { useEditorKeys } from "./keys";
import { closeContextMenu, useContextMenu, useMenuBar } from "./menus";
import { useAnnouncement } from "./announce";
import { useEditor } from "./state";
import { HintBubble } from "../components/Hints";
import { dragged, draggingOut } from "../library/dragOut";
import { LibraryView } from "../library/LibraryView";
import { useMainView } from "../library/state";
import { UndoNotice } from "./UndoNotice";
import { Workspace } from "./Workspace";
import { VideoStage } from "./VideoStage";
import { GifDialog } from "./GifDialog";
import { CombineDialog } from "./CombineDialog";
import { TemplateDialogs } from "./TemplatePicker";
import { dropOnCanvas, dropOnZone } from "./templateActions";
import { canvasPointAt, useDropPoint } from "./templateUi";
import { EditTextDialog } from "./EditTextDialog";
import { QrDialogs } from "./QrDialogs";

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
  const trayItems = useEditor((s) => s.documents.filter((d) => d.shown).length);
  const [dropping, setDropping] = useState(false);

  useEffect(() => {
    void useEditor.getState().boot();
  }, []);

  // Themes and templates opened with XcreenShot start their import here (APP-001).
  useEffect(() => {
    const take = () =>
      void run<{ paths: string[] }>("app.take_opening", { kind: "editor" })
        .then((r) => (r.paths.length > 0 ? openFiles(r.paths) : undefined))
        .catch(() => undefined);
    take();
    const off = listen<{ kind: string; payload: { kind?: string } }>("xs://editor", (e) => {
      if (e.payload.kind === "opening" && e.payload.payload.kind === "editor") take();
    });
    return () => void off.then((f) => f());
  }, []);

  // Files dropped on the window open as documents (EDT-105). Floating images on the
  // canvas arrive with M2.
  useEffect(() => {
    let off: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent((e) => {
        // XcreenShot's own items dragged out (TRY-015) do not open here again; on a
        // template's zone they fill it (EDT-098), on the canvas of another document
        // they become floating images (TRY-014).
        if (draggingOut()) {
          setDropping(false);
          if (e.payload.type === "over")
            useDropPoint.setState({ at: canvasPointAt(e.payload.position) });
          else if (e.payload.type === "leave") useDropPoint.setState({ at: null });
          else if (e.payload.type === "drop") {
            useDropPoint.setState({ at: null });
            const sources = dragged()?.docs ?? [];
            if (!dropOnZone(e.payload.paths, e.payload.position))
              dropOnCanvas(e.payload.paths, e.payload.position, sources);
          }
          return;
        }
        if (e.payload.type === "enter" || e.payload.type === "over") setDropping(true);
        else if (e.payload.type === "leave") setDropping(false);
        else if (e.payload.type === "drop") {
          setDropping(false);
          // On the Library, files and folders are imported (LIB-024).
          if (useMainView.getState().view === "library") {
            askImport(e.payload.paths);
            return;
          }
          // On a template's zone, an image fills it (EDT-098).
          if (dropOnZone(e.payload.paths, e.payload.position)) return;
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
        // The canvas fills the workspace; the bars, the panel and the recent tray (which
        // belongs to the Editor view, LIB-001) float over it.
        <Workspace>
          {(insets) =>
            doc?.video ? (
              // A video plays in the system's player (ADR 0008).
              <VideoStage doc={doc} insets={insets} />
            ) : doc ? (
              <Canvas key={doc.documentId} doc={doc} insets={insets} />
            ) : (
              <Empty dropping={dropping} />
            )
          }
        </Workspace>
      )}
      <UndoNotice />
      {view === "editor" && <PendingShare />}
      {/* One hint at a time, the first time (APP-014). */}
      {view === "editor" && <HintBubble trayItems={trayItems} />}
      {toast && (
        <div className="editor-toast" role="status" key={toast}>
          {toast}
        </div>
      )}
      {dropping && view === "editor" && doc && (
        <div className="drop-hint">{t("ui.editor.drop_hint")}</div>
      )}
      {dropping && view === "library" && (
        <div className="drop-hint">{t("ui.library.import.drop_hint")}</div>
      )}
      <Dialogs />
      <RedactBatch />
      <EditTextDialog />
      <QrDialogs />
      <GifDialog />
      <CombineDialog />
      <TemplateDialogs />
      <QuickView />
      <TagDialogs />
      <ShortcutSheet />
      <MoveComputer />
      <EditorContextMenu />
      <div className="sr-only" role="status" aria-live="polite">
        {announcement}
      </div>
    </div>
  );
}
