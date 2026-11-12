// Status bar (EDT-011): the Recent button (TRY-001), image size, pointer position and
// zoom.
import { useTranslation } from "react-i18next";
import { Menu } from "../components/Menu";
import { useTray } from "../library/state";
import { hint, useShortcut } from "./tools";
import { Icon } from "./icons";
import { outside } from "./objects";
import { requestZoom, useEditor, usePointer } from "./state";
import { useDraft } from "./useDraft";
import { hasEffects } from "./finalPreview";
import { parseZoom, ZOOM_STEPS } from "./view";
import { openImageDialog } from "./imageMenu";

function PointerPosition({ width, height }: { width: number; height: number }) {
  const point = usePointer((s) => s.point);
  const { t } = useTranslation();
  if (!point) return <span className="status-item muted" />;
  const x = Math.floor(point[0]);
  const y = Math.floor(point[1]);
  if (x < 0 || y < 0 || x >= width || y >= height) return <span className="status-item muted" />;
  return (
    <span className="status-item" title={t("ui.editor.pointer")}>
      {x}, {y}
    </span>
  );
}

/** Objects left outside the canvas, e.g. after cropping, with Select (EDT-049). */
function OutsideObjects() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  if (!doc) return null;
  const ids = outside(doc.objects, doc.canvas.width, doc.canvas.height);
  if (ids.length === 0) return null;
  return (
    <span className="status-item status-outside">
      {t("ui.editor.outside", { count: ids.length })}{" "}
      <button
        type="button"
        className="btn small"
        onClick={() => useEditor.getState().setSelection(ids)}
      >
        {t("ui.editor.select_outside")}
      </button>
    </span>
  );
}

/** Shows or hides the recent tray; a dot when captures arrived while hidden (TRY-001). */
function RecentButton() {
  const { t } = useTranslation();
  const visible = useTray((s) => s.visible);
  const fresh = useTray((s) => s.fresh);
  const toggle = useTray((s) => s.toggle);
  const key = useShortcut("editor.tray");
  const name = t("ui.editor.recent");
  return (
    <button
      type="button"
      className="btn small recent-toggle"
      aria-pressed={visible}
      aria-label={fresh && !visible ? t("ui.editor.recent_new") : name}
      title={hint(t("menu.tray"), key)}
      onClick={toggle}
    >
      <Icon name="recent" />
      <span>{name}</span>
      {fresh && !visible && <span className="recent-dot" aria-hidden="true" />}
    </button>
  );
}

export function StatusBar() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const fxSize = useEditor((s) => s.fxSize);
  const zoom = useEditor((s) => (s.active ? s.views[s.active]?.zoom : undefined));
  const [draft, setDraft] = useDraft(zoom ? `${Math.round(zoom * 10) / 10} %` : "");
  if (!doc)
    return (
      <footer className="status-bar">
        <RecentButton />
      </footer>
    );

  const typed = () => {
    const z = parseZoom(draft);
    if (z !== null) requestZoom("percent", z);
    setDraft(null);
  };

  return (
    <footer className="status-bar">
      <RecentButton />
      {/* A click opens Resize image (EDT-069); the resolution opens its dialog (EDT-077). */}
      <button
        type="button"
        className="status-item status-button"
        title={t("menu.resize_image")}
        aria-label={`${t("ui.editor.prop.size")}: ${doc.canvas.width} × ${doc.canvas.height} px`}
        onClick={() => openImageDialog("resize")}
      >
        {doc.canvas.width} × {doc.canvas.height} px
        {fxSize &&
          hasEffects(doc) &&
          (fxSize[0] !== doc.canvas.width || fxSize[1] !== doc.canvas.height) && (
            // The result is bigger (or smaller) with its effects (EDT-053).
            <span className="status-fx">
              {" → "}
              {t("ui.editor.fx.with_effects", { w: fxSize[0], h: fxSize[1] })}
            </span>
          )}
      </button>
      <button
        type="button"
        className="status-item status-button"
        title={t("menu.resolution")}
        onClick={() => openImageDialog("resolution")}
      >
        {t("ui.editor.dpi", { dpi: Math.round(doc.dpi) })}
      </button>
      {doc.canvas.cornerRadius > 0 && (
        // The radius of the rounded corners (EDT-075).
        <button
          type="button"
          className="status-item status-button"
          title={t("menu.corners")}
          onClick={() => openImageDialog("corners")}
        >
          {t("ui.editor.image.status_radius", { r: Math.round(doc.canvas.cornerRadius) })}
        </button>
      )}
      <PointerPosition width={doc.canvas.width} height={doc.canvas.height} />
      <OutsideObjects />
      <span className="grow" />
      <div className="zoom-control" role="group" aria-label={t("ui.editor.zoom")}>
        <button
          type="button"
          className="btn icon small"
          aria-label={t("ui.editor.zoom_out")}
          title={t("ui.editor.zoom_out")}
          onClick={() => requestZoom("out")}
        >
          <Icon name="minus" />
        </button>
        <input
          type="text"
          className="zoom-field"
          value={draft}
          aria-label={t("ui.editor.zoom")}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={typed}
          onKeyDown={(e) => e.key === "Enter" && typed()}
        />
        <button
          type="button"
          className="btn icon small"
          aria-label={t("ui.editor.zoom_in")}
          title={t("ui.editor.zoom_in")}
          onClick={() => requestZoom("in")}
        >
          <Icon name="plus" />
        </button>
        <Menu
          triggerClass="btn icon small"
          ariaLabel={t("ui.editor.zoom_menu")}
          title={t("ui.editor.zoom_menu")}
          side="top"
          align="end"
          label={<Icon name="chevron" />}
          chevron={false}
          items={[
            {
              id: "actual",
              label: t("ui.editor.zoom_actual"),
              onSelect: () => requestZoom("percent", 100),
            },
            { id: "fit", label: t("ui.editor.zoom_fit"), onSelect: () => requestZoom("fit") },
            {
              id: "original",
              label: t("ui.editor.zoom_original"),
              onSelect: () => requestZoom("original"),
            },
            ...ZOOM_STEPS.map((z, i) => ({
              id: `z${z}`,
              label: `${z} %`,
              onSelect: () => requestZoom("percent", z),
              separator: i === 0,
            })),
          ]}
        />
      </div>
    </footer>
  );
}
