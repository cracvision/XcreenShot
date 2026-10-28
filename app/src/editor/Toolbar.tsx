// Editor toolbar (EDT-001, EDT-003): Capture, the tools of this build, Undo and Redo
// (they stay with the menus: Carlos, 2026-10-28), and the share area (Copy all,
// Share ▾). Tooltips show the current shortcut (APP-014).
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "../components/LanguageToggle";
import { MenuButton } from "../components/MenuButton";
import { run } from "../lib/commands";
import {
  copyAll,
  exportImage,
  openCapturePanel,
  openFiles,
  redo,
  saveActive,
  saveAs,
  selectTool,
  undo,
} from "./actions";
import { Icon } from "./icons";
import { useEditor, type Tool } from "./state";
import { toolbarMenu } from "./menus";
import { useSetting } from "../store";
import { hint, useShortcut } from "./tools";

function ToolButton({ id, active }: { id: Tool; active: boolean }) {
  const { t } = useTranslation();
  const name = t(`ui.editor.tool.${id}`);
  const shortcut = useShortcut(`editor.tool_${id}`);
  return (
    <button
      type="button"
      className="tb-tool"
      aria-pressed={active}
      title={hint(name, shortcut)}
      onClick={() => selectTool(id)}
    >
      <Icon name={id} />
      <span className="tb-label">{name}</span>
    </button>
  );
}

export function Toolbar() {
  const { t } = useTranslation();
  const tool = useEditor((s) => s.tool);
  const tools = useEditor((s) => s.tools);
  const doc = useEditor((s) => s.doc);
  const hasDoc = doc !== null;
  const labels = useSetting("editor.toolbarLabels", true);
  const keys = {
    undo: useShortcut("editor.undo"),
    redo: useShortcut("editor.redo"),
    open: useShortcut("editor.open"),
    save: useShortcut("editor.save"),
    copyAll: useShortcut("editor.copy_all"),
  };

  return (
    <div
      className={`editor-toolbar${labels ? "" : " icons-only"}`}
      role="toolbar"
      aria-label={t("ui.editor.toolbar")}
      onContextMenu={(e) => {
        e.preventDefault();
        void toolbarMenu();
      }}
    >
      <button type="button" className="btn primary tb-btn" onClick={openCapturePanel}>
        <Icon name="capture" />
        <span className="tb-label">{t("ui.editor.capture")}</span>
      </button>
      <div className="tb-group" role="group" aria-label={t("ui.editor.tools")}>
        {tools.map((id: Tool) => (
          <ToolButton key={id} id={id} active={tool === id} />
        ))}
      </div>
      <div className="tb-group">
        <button
          type="button"
          className="btn icon"
          disabled={!doc?.canUndo}
          title={hint(
            doc?.undoLabel
              ? t("ui.editor.undo_named", { label: doc.undoLabel })
              : t("ui.editor.undo"),
            keys.undo,
          )}
          aria-label={t("ui.editor.undo")}
          onClick={undo}
        >
          <Icon name="undo" />
        </button>
        <button
          type="button"
          className="btn icon"
          disabled={!doc?.canRedo}
          title={hint(
            doc?.redoLabel
              ? t("ui.editor.redo_named", { label: doc.redoLabel })
              : t("ui.editor.redo"),
            keys.redo,
          )}
          aria-label={t("ui.editor.redo")}
          onClick={redo}
        >
          <Icon name="redo" />
        </button>
      </div>
      <div className="tb-spacer" />
      <div className="tb-group">
        <button
          type="button"
          className="btn tb-btn"
          title={hint(t("ui.editor.open"), keys.open)}
          onClick={() => void openFiles()}
        >
          <Icon name="open" />
          <span className="tb-label tb-wide">{t("ui.editor.open")}</span>
        </button>
        <button
          type="button"
          className="btn tb-btn"
          disabled={!hasDoc}
          title={hint(t("ui.editor.save"), keys.save)}
          onClick={() => void saveActive()}
        >
          <Icon name="save" />
          <span className="tb-label tb-wide">{t("ui.editor.save")}</span>
        </button>
      </div>
      <div className="tb-group share" role="group" aria-label={t("ui.editor.share_area")}>
        <button
          type="button"
          className="btn tb-btn"
          disabled={!hasDoc}
          title={hint(t("ui.editor.copy_all"), keys.copyAll)}
          onClick={() => void copyAll()}
        >
          <Icon name="copy" />
          <span className="tb-label tb-wide">{t("ui.editor.copy_all")}</span>
        </button>
        <MenuButton
          className="btn tb-btn"
          disabled={!hasDoc}
          title={t("ui.editor.share")}
          label={
            <>
              <Icon name="share" />
              <span className="tb-label tb-wide">{t("ui.editor.share")}</span>
              <Icon name="chevron" />
            </>
          }
          items={[
            { label: t("ui.editor.share_file"), onSelect: () => void exportImage() },
            { label: t("ui.editor.share_clipboard"), onSelect: () => void copyAll() },
            { label: t("ui.editor.save_as"), onSelect: () => void saveAs(), separator: true },
          ]}
        />
      </div>
      <LanguageToggle />
      <button
        type="button"
        className="btn icon"
        title={t("ui.editor.preferences")}
        aria-label={t("ui.editor.preferences")}
        onClick={() => void run("settings.open")}
      >
        <Icon name="settings" />
      </button>
    </div>
  );
}
