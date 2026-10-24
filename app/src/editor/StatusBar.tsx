// Status bar (EDT-011): image size, pointer position and zoom.
import { useTranslation } from "react-i18next";
import { MenuButton } from "../components/MenuButton";
import { Icon } from "./icons";
import { requestZoom, useEditor, usePointer } from "./state";
import { useDraft } from "./useDraft";
import { parseZoom, ZOOM_STEPS } from "./view";

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

export function StatusBar() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const zoom = useEditor((s) => (s.active ? s.views[s.active]?.zoom : undefined));
  const [draft, setDraft] = useDraft(zoom ? `${Math.round(zoom * 10) / 10} %` : "");
  if (!doc) return <footer className="status-bar" />;

  const typed = () => {
    const z = parseZoom(draft);
    if (z !== null) requestZoom("percent", z);
    setDraft(null);
  };

  return (
    <footer className="status-bar">
      <span className="status-item" title={t("ui.editor.prop.size")}>
        {doc.canvas.width} × {doc.canvas.height} px
      </span>
      <PointerPosition width={doc.canvas.width} height={doc.canvas.height} />
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
        <MenuButton
          className="btn icon small"
          ariaLabel={t("ui.editor.zoom_menu")}
          title={t("ui.editor.zoom_menu")}
          up
          label=<Icon name="chevron" />
          items={[
            { label: t("ui.editor.zoom_actual"), onSelect: () => requestZoom("percent", 100) },
            { label: t("ui.editor.zoom_fit"), onSelect: () => requestZoom("fit") },
            { label: t("ui.editor.zoom_original"), onSelect: () => requestZoom("original") },
            ...ZOOM_STEPS.map((z, i) => ({
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
