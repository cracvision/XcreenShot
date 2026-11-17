// The options of a guide's export before choosing where it goes: its pictures one by
// one (SMT-034: the format, the size, a saved effect and the names, as the batch
// export, EDT-113) and the PDF (SMT-033: the paper and the orientation).
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import {
  exportGuideImages,
  exportGuidePdf,
  useGuideImages,
  useGuidePdf,
  type GuideImagesOptions,
  type GuidePdfOptions,
} from "./guideExport";

interface SavedEffect {
  id: string;
  name: string;
}

/** The PDF's paper and orientation (SMT-033). */
export function GuidePdfDialog() {
  const { t } = useTranslation();
  const ask = useGuidePdf((s) => s.ask);
  const [o, setO] = useState<GuidePdfOptions>({ paper: "auto", orientation: "portrait" });
  if (!ask) return null;
  const close = () => useGuidePdf.setState({ ask: null });
  return (
    <Dialog
      title={t("ui.guide.pdf.title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn primary" onClick={() => void exportGuidePdf(o)}>
            {t("ui.guide.images.export")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.common.cancel")}
          </button>
        </>
      }
    >
      <div className="prop-field">
        <label htmlFor="guide-pdf-paper">{t("ui.guide.pdf.paper")}</label>
        <select
          id="guide-pdf-paper"
          data-autofocus
          value={o.paper}
          onChange={(e) => setO({ ...o, paper: e.target.value as GuidePdfOptions["paper"] })}
        >
          <option value="auto">{t("ui.guide.pdf.paper_auto")}</option>
          <option value="a4">A4</option>
          <option value="letter">{t("ui.guide.pdf.paper_letter")}</option>
        </select>
      </div>
      <div className="prop-field">
        <label htmlFor="guide-pdf-orientation">{t("ui.guide.pdf.orientation")}</label>
        <select
          id="guide-pdf-orientation"
          value={o.orientation}
          onChange={(e) =>
            setO({ ...o, orientation: e.target.value as GuidePdfOptions["orientation"] })
          }
        >
          <option value="portrait">{t("ui.guide.pdf.portrait")}</option>
          <option value="landscape">{t("ui.guide.pdf.landscape")}</option>
        </select>
      </div>
    </Dialog>
  );
}

export function GuideImagesDialog() {
  const ask = useGuideImages((s) => s.ask);
  if (!ask) return null;
  return <GuideImagesForm key={ask.name} name={ask.name} />;
}

function GuideImagesForm({ name }: { name: string }) {
  const { t } = useTranslation();
  const tt = (k: string, o: Record<string, unknown> = {}) => t(`ui.guide.images.${k}`, o);
  const [o, setO] = useState<GuideImagesOptions>({
    imageFormat: "png",
    scale: 100,
    effects: "",
    base: name,
    start: 1,
    withText: false,
  });
  const [effects, setEffects] = useState<SavedEffect[]>([]);
  useEffect(() => {
    void run<{ effects: SavedEffect[] }>("editor.saved_effects")
      .then((r) => setEffects(r.effects))
      .catch(() => undefined);
  }, []);
  const set = (p: Partial<GuideImagesOptions>) => setO((x) => ({ ...x, ...p }));
  const badScale = !(o.scale >= 25 && o.scale <= 400);
  const example = `${o.base.trim() || name}_${String(Math.max(0, o.start)).padStart(2, "0")}.${o.imageFormat}`;
  const close = () => useGuideImages.setState({ ask: null });
  return (
    <Dialog
      title={tt("title")}
      onCancel={close}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={badScale}
            onClick={() => void exportGuideImages(o)}
          >
            {tt("export")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.common.cancel")}
          </button>
        </>
      }
    >
      <div className="prop-field">
        <label htmlFor="guide-img-format">{tt("format")}</label>
        <select
          id="guide-img-format"
          data-autofocus
          value={o.imageFormat}
          onChange={(e) => set({ imageFormat: e.target.value as "png" | "jpg" })}
        >
          <option value="png">PNG</option>
          <option value="jpg">JPG</option>
        </select>
      </div>
      <div className="prop-field">
        <label htmlFor="guide-img-scale">{tt("scale")}</label>
        <input
          id="guide-img-scale"
          type="number"
          min={25}
          max={400}
          value={o.scale}
          aria-invalid={badScale}
          aria-describedby={badScale ? "guide-img-scale-error" : undefined}
          onChange={(e) => set({ scale: Number(e.target.value) })}
        />
        {badScale && (
          <span id="guide-img-scale-error" className="panel-note" role="alert">
            {tt("scale_range")}
          </span>
        )}
      </div>
      <div className="prop-field">
        <label htmlFor="guide-img-effects">{tt("effects")}</label>
        <select
          id="guide-img-effects"
          value={o.effects}
          onChange={(e) => set({ effects: e.target.value })}
        >
          <option value="">{tt("no_effects")}</option>
          {effects.map((e) => (
            <option key={e.id} value={e.id}>
              {e.name}
            </option>
          ))}
        </select>
      </div>
      <div className="prop-field">
        <label htmlFor="guide-img-base">{tt("name")}</label>
        <input
          id="guide-img-base"
          type="text"
          value={o.base}
          onChange={(e) => set({ base: e.target.value })}
        />
      </div>
      <div className="prop-field">
        <label htmlFor="guide-img-start">{tt("start")}</label>
        <input
          id="guide-img-start"
          type="number"
          min={0}
          value={o.start}
          onChange={(e) => set({ start: Math.max(0, Math.floor(Number(e.target.value) || 0)) })}
        />
      </div>
      <p className="muted">{tt("example", { name: example })}</p>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={o.withText}
          onChange={(e) => set({ withText: e.target.checked })}
        />
        {tt("with_text")}
      </label>
    </Dialog>
  );
}
