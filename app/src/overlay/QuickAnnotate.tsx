// Quick annotation's bar (CAP-057): the tools, color and width, undo and redo, and
// Copy, Open in the editor and Cancel.
import { useTranslation } from "react-i18next";
import { COLORS, TOOLS, WIDTHS, type Tool } from "./quickMarks";

/** The compact bar of quick annotation (CAP-057), placed like the capture bar. */
export function QuickBar({
  style,
  tool,
  color,
  width,
  ellipse,
  canUndo,
  canRedo,
  onTool,
  onColor,
  onWidth,
  onEllipse,
  onUndo,
  onRedo,
  onCopy,
  onEditor,
  onCancel,
}: {
  style: React.CSSProperties;
  tool: Tool;
  color: string;
  width: number;
  ellipse: boolean;
  canUndo: boolean;
  canRedo: boolean;
  onTool: (t: Tool) => void;
  onColor: (c: string) => void;
  onWidth: (w: number) => void;
  onEllipse: (e: boolean) => void;
  onUndo: () => void;
  onRedo: () => void;
  onCopy: () => void;
  onEditor: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="overlay-bar quick" role="toolbar" aria-label={t("ui.quick.bar")} style={style}>
      {TOOLS.map((k, i) => (
        <button
          key={k}
          type="button"
          className={`overlay-btn${tool === k ? " on" : ""}`}
          aria-pressed={tool === k}
          title={`${t(`ui.quick.tool_${k}`)} (${i + 1})`}
          onClick={() => onTool(k)}
        >
          {t(`ui.quick.tool_${k}`)} <kbd>{i + 1}</kbd>
        </button>
      ))}
      {tool === "shape" && (
        <button type="button" className="overlay-btn" onClick={() => onEllipse(!ellipse)}>
          {ellipse ? t("ui.quick.ellipse") : t("ui.quick.rectangle")}
        </button>
      )}
      <span className="overlay-colors" role="radiogroup" aria-label={t("ui.quick.color")}>
        {COLORS.map((c) => (
          <button
            key={c}
            type="button"
            role="radio"
            aria-checked={color === c}
            aria-label={c}
            className="overlay-color"
            style={{ background: c }}
            onClick={() => onColor(c)}
          />
        ))}
      </span>
      <select
        className="overlay-select"
        aria-label={t("ui.quick.width")}
        value={width}
        onChange={(e) => onWidth(Number(e.target.value))}
      >
        {WIDTHS.map((w) => (
          <option key={w} value={w}>
            {w} px
          </option>
        ))}
      </select>
      <button
        type="button"
        className="overlay-btn"
        disabled={!canUndo}
        onClick={onUndo}
        title="Ctrl+Z"
      >
        {t("ui.quick.undo")}
      </button>
      <button
        type="button"
        className="overlay-btn"
        disabled={!canRedo}
        onClick={onRedo}
        title="Ctrl+Shift+Z"
      >
        {t("ui.quick.redo")}
      </button>
      <button type="button" className="overlay-btn primary" onClick={onCopy} title="Enter · Ctrl+C">
        {t("ui.quick.copy")} <kbd>Enter</kbd>
      </button>
      <button type="button" className="overlay-btn" onClick={onEditor} title="Ctrl+E">
        {t("ui.quick.editor")}
      </button>
      <button type="button" className="overlay-btn" onClick={onCancel} title="Esc">
        {t("ui.quick.cancel")}
      </button>
    </div>
  );
}
