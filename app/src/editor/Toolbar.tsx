// Editor toolbar (EDT-001, EDT-003): Capture, the tools of this build, and the share
// area (Copy all, Share ▾). Undo and Redo are here until the menus arrive (build 4c).
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

const TOOL_KEYS: Record<string, string> = { move: "V", arrow: "A" };

export function Toolbar() {
  const { t } = useTranslation();
  const tool = useEditor((s) => s.tool);
  const tools = useEditor((s) => s.tools);
  const doc = useEditor((s) => s.doc);
  const hasDoc = doc !== null;

  const hint = (label: string, key?: string) => (key ? `${label} (${key})` : label);

  return (
    <div className="editor-toolbar" role="toolbar" aria-label={t("ui.editor.toolbar")}>
      <button type="button" className="btn primary tb-btn" onClick={openCapturePanel}>
        <Icon name="capture" />
        <span className="tb-label">{t("ui.editor.capture")}</span>
      </button>
      <div className="tb-group" role="group" aria-label={t("ui.editor.tools")}>
        {tools.map((id: Tool) => {
          const name = t(`ui.editor.tool.${id}`);
          return (
            <button
              key={id}
              type="button"
              className="tb-tool"
              aria-pressed={tool === id}
              title={hint(name, TOOL_KEYS[id])}
              onClick={() => selectTool(id)}
            >
              <Icon name={id as "move" | "arrow"} />
              <span className="tb-label">{name}</span>
            </button>
          );
        })}
      </div>
      <div className="tb-group">
        <button
          type="button"
          className="btn icon"
          disabled={!doc?.canUndo}
          title={
            doc?.undoLabel
              ? t("ui.editor.undo_named", { label: doc.undoLabel })
              : t("ui.editor.undo")
          }
          aria-label={t("ui.editor.undo")}
          onClick={undo}
        >
          <Icon name="undo" />
        </button>
        <button
          type="button"
          className="btn icon"
          disabled={!doc?.canRedo}
          title={
            doc?.redoLabel
              ? t("ui.editor.redo_named", { label: doc.redoLabel })
              : t("ui.editor.redo")
          }
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
          title={t("ui.editor.open")}
          onClick={() => void openFiles()}
        >
          <Icon name="open" />
          <span className="tb-label tb-wide">{t("ui.editor.open")}</span>
        </button>
        <button
          type="button"
          className="btn tb-btn"
          disabled={!hasDoc}
          title={t("ui.editor.save")}
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
          title={t("ui.editor.copy_all")}
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
