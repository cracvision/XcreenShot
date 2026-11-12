// File > Export (EDT-110 to EDT-112): the format, its options and the size the file
// will have; then the system dialog asks where. Remembers the format and its options.
import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import {
  EXTENSIONS,
  FORMATS,
  ICO_SIZES,
  UNAVAILABLE,
  fileSize,
  lastExportFormat,
  rememberExport,
  rememberedOptions,
  useExportDialog,
  type ExportFormat,
  type ExportOptions,
} from "./exportOptions";
import { useEditor } from "./state";

function Check(props: { label: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="panel-check">
      <input
        type="checkbox"
        checked={props.checked}
        onChange={(e) => props.onChange(e.target.checked)}
      />
      {props.label}
    </label>
  );
}

function Num(props: {
  id: string;
  label: string;
  value: number;
  min: number;
  max: number;
  suffix: string;
  onChange: (v: number) => void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  return (
    <div className="prop-field">
      <label htmlFor={props.id}>{props.label}</label>
      <span className="image-number">
        <input
          id={props.id}
          type="text"
          inputMode="numeric"
          value={draft ?? String(props.value)}
          onChange={(e) => {
            setDraft(e.target.value);
            const v = Number(e.target.value);
            if (Number.isFinite(v) && v >= props.min && v <= props.max) props.onChange(v);
          }}
          onBlur={() => setDraft(null)}
        />
        <span className="muted">{props.suffix}</span>
      </span>
    </div>
  );
}

function Select<T extends string>(props: {
  id: string;
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="prop-field">
      <label htmlFor={props.id}>{props.label}</label>
      <select
        id={props.id}
        value={props.value}
        onChange={(e) => props.onChange(e.target.value as T)}
      >
        {props.options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}

export function ExportDialog() {
  const ids = useExportDialog((s) => s.ids);
  if (!ids || ids.length === 0) return null;
  return <ExportForm key={ids.join(",")} ids={ids} />;
}

function ExportForm({ ids }: { ids: string[] }) {
  const { t, i18n } = useTranslation();
  const name = useEditor((s) => s.documents.find((d) => d.documentId === ids[0])?.name ?? "");
  const [format, setFormat] = useState<ExportFormat>(lastExportFormat);
  const [o, setO] = useState<ExportOptions>(() => rememberedOptions(lastExportFormat()));
  // The measured size, for the settings it was measured with.
  const [measured, setMeasured] = useState<{ key: string; bytes: number } | null>(null);
  const key = JSON.stringify([format, o, ids]);
  const size = measured?.key === key ? measured.bytes : null;
  const set = (changes: Partial<ExportOptions>) => setO((prev) => ({ ...prev, ...changes }));
  const close = () => useExportDialog.setState({ ids: null });
  const many = ids.length > 1;
  const pick = (f: ExportFormat) => {
    setFormat(f);
    setO(rememberedOptions(f));
  };
  // The size the file would have, again after each change (EDT-112).
  useEffect(() => {
    let alive = true;
    const timer = setTimeout(() => {
      run<{ bytes: number }>("document.export", {
        ...(many ? { documentIds: ids } : { documentId: ids[0] }),
        format,
        options: o,
        estimate: true,
      })
        .then((r) => alive && setMeasured({ key, bytes: r.bytes }))
        .catch(() => undefined);
    }, 250);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [format, o, ids, many, key]);

  const go = async () => {
    try {
      const path = await save({
        defaultPath: `${name}.${EXTENSIONS[format][0]}`,
        filters: [{ name: t(`ui.editor.export.format_${format}`), extensions: EXTENSIONS[format] }],
      });
      if (!path) return;
      close();
      rememberExport(format, o);
      // The system dialog already asked before replacing a file.
      await run("document.export", {
        ...(many ? { documentIds: ids } : { documentId: ids[0] }),
        format,
        path,
        options: o,
        onConflict: "replace",
      });
      useEditor.getState().showToast(t("ui.editor.exported"));
    } catch (e) {
      useEditor.getState().showError(e);
    }
  };

  const opt = (key: string) => t(`ui.editor.export.${key}`);
  let fields: React.ReactNode;
  switch (format) {
    case "jpg":
      fields = (
        <Num
          id="ex-quality"
          label={opt("quality")}
          value={o.quality}
          min={1}
          max={100}
          suffix=""
          onChange={(quality) => set({ quality })}
        />
      );
      break;
    case "png":
      fields = (
        <>
          <Select
            id="ex-png"
            label={opt("compression")}
            value={o.pngCompression}
            options={(["fast", "max"] as const).map((v) => ({ value: v, label: opt(`png_${v}`) }))}
            onChange={(pngCompression) => set({ pngCompression })}
          />
          <Check
            label={opt("colors256")}
            checked={o.pngColors256}
            onChange={(pngColors256) => set({ pngColors256 })}
          />
          <Check
            label={opt("color_profile")}
            checked={o.colorProfile}
            onChange={(colorProfile) => set({ colorProfile })}
          />
        </>
      );
      break;
    case "gif":
      fields = (
        <Check
          label={opt("gif_transparent")}
          checked={o.gifTransparent}
          onChange={(gifTransparent) => set({ gifTransparent })}
        />
      );
      break;
    case "tiff":
      fields = (
        <Select
          id="ex-tiff"
          label={opt("compression")}
          value={o.tiffCompression}
          options={(["none", "lzw", "zip"] as const).map((v) => ({
            value: v,
            label: opt(`tiff_${v}`),
          }))}
          onChange={(tiffCompression) => set({ tiffCompression })}
        />
      );
      break;
    case "pdf":
      fields = (
        <>
          <Select
            id="ex-page"
            label={opt("page")}
            value={o.pdfPage}
            options={(["auto", "a4", "letter", "legal", "a3"] as const).map((v) => ({
              value: v,
              label: opt(`page_${v}`),
            }))}
            onChange={(pdfPage) => set({ pdfPage })}
          />
          {o.pdfPage !== "auto" && (
            <>
              <Select
                id="ex-orient"
                label={opt("orientation")}
                value={o.pdfOrientation}
                options={(["auto", "portrait", "landscape"] as const).map((v) => ({
                  value: v,
                  label: opt(`orient_${v}`),
                }))}
                onChange={(pdfOrientation) => set({ pdfOrientation })}
              />
              <Num
                id="ex-margin"
                label={opt("margin")}
                value={o.pdfMargin}
                min={0}
                max={100}
                suffix="mm"
                onChange={(pdfMargin) => set({ pdfMargin })}
              />
              <Check
                label={opt("fit_width")}
                checked={o.pdfFitWidth}
                onChange={(pdfFitWidth) => set({ pdfFitWidth })}
              />
              <Check
                label={opt("single_page")}
                checked={o.pdfSinglePage}
                onChange={(pdfSinglePage) => set({ pdfSinglePage })}
              />
            </>
          )}
          <Check
            label={opt("lossless")}
            checked={o.pdfLossless}
            onChange={(pdfLossless) => set({ pdfLossless })}
          />
          {!o.pdfLossless && (
            <Num
              id="ex-pdfq"
              label={opt("quality")}
              value={o.quality}
              min={1}
              max={100}
              suffix=""
              onChange={(quality) => set({ quality })}
            />
          )}
        </>
      );
      break;
    case "ico":
      fields = (
        <fieldset className="ico-sizes">
          <legend>{opt("ico_sizes")}</legend>
          {ICO_SIZES.map((s) => (
            <Check
              key={s}
              label={`${s} px`}
              checked={o.icoSizes.includes(s)}
              onChange={(on) => {
                const next = on ? [...o.icoSizes, s] : o.icoSizes.filter((v) => v !== s);
                if (next.length > 0) set({ icoSizes: next.sort((a, b) => a - b) });
              }}
            />
          ))}
        </fieldset>
      );
      break;
    default:
      fields = null;
  }

  return (
    <Dialog
      title={t(many ? "ui.editor.export.title_many" : "ui.editor.export.title", {
        count: ids.length,
      })}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn primary" onClick={() => void go()}>
            {t("ui.editor.export.choose")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form">
        <div className="prop-field">
          <label htmlFor="ex-format">{opt("format")}</label>
          <select
            id="ex-format"
            data-autofocus=""
            value={format}
            onChange={(e) => pick(e.target.value as ExportFormat)}
          >
            {FORMATS.filter((f) => !many || f === "pdf" || f === "tiff").map((f) => (
              <option key={f} value={f}>
                {t(`ui.editor.export.format_${f}`)}
              </option>
            ))}
            {!many &&
              UNAVAILABLE.map((f) => (
                <option key={f} value={f} disabled title={t("ui.editor.export.unavailable")}>
                  {t(`ui.editor.export.format_${f}`)} — {t("ui.editor.export.unavailable_short")}
                </option>
              ))}
          </select>
        </div>
        {fields}
        <Num
          id="ex-scale"
          label={opt("scale")}
          value={o.scale}
          min={25}
          max={400}
          suffix="%"
          onChange={(scale) => set({ scale })}
        />
      </div>
      <p className="muted" aria-live="polite">
        {size === null
          ? t("ui.editor.export.measuring")
          : t("ui.editor.export.size", { size: fileSize(size, i18n.language) })}
      </p>
    </Dialog>
  );
}
