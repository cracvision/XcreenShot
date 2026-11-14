// File > Print (EDT-114): our own preview with paper, orientation, margins, size,
// position and whole document or selection; Print hands the pages to the system's
// print dialog. The paper setup is remembered.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { useEditor } from "./state";
import { usePrintDialog } from "./printState";

interface Setup {
  paper: "a4" | "letter" | "legal" | "a3";
  orientation: "auto" | "portrait" | "landscape";
  margin: number;
  size: "fit" | "actual" | "percent";
  percent: number;
  position: "center" | "topLeft";
}

interface PrintPage {
  png: string;
  paper: { w: number; h: number };
  at: { x: number; y: number };
  size: { w: number; h: number };
}

const KEY = "xs.print";
const DEFAULT: Setup = {
  paper: "a4",
  orientation: "auto",
  margin: 10,
  size: "fit",
  percent: 100,
  position: "center",
};

function remembered(): Setup {
  try {
    return { ...DEFAULT, ...(JSON.parse(localStorage.getItem(KEY) ?? "{}") as Partial<Setup>) };
  } catch {
    return DEFAULT;
  }
}

function remember(s: Setup): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    // Not remembered: no harm.
  }
}

export function PrintDialog() {
  const open = usePrintDialog((s) => s.open);
  const paths = usePrintDialog((s) => s.paths);
  const doc = useEditor((s) => s.doc);
  if (!open || (!doc && !paths)) return null;
  return <PrintForm key={paths ? paths.join("|") : doc?.documentId} paths={paths} />;
}

function PrintForm({ paths }: { paths?: string[] | undefined }) {
  const { t } = useTranslation();
  const editing = useEditor((s) => s.doc);
  // Files from the library (LIB-017) print without the document shown.
  const doc = paths ? null : editing;
  const [s, setS] = useState<Setup>(remembered);
  const [what, setWhat] = useState<"all" | "selection">("all");
  const [result, setResult] = useState<{ key: string; pages: PrintPage[] } | null>(null);
  const [printing, setPrinting] = useState(false);
  const set = (changes: Partial<Setup>) => setS((prev) => ({ ...prev, ...changes }));
  const close = () => usePrintDialog.setState({ open: false, paths: undefined });
  const tt = (k: string, o?: Record<string, unknown>) => t(`ui.editor.print.${k}`, o ?? {});
  const sel = doc?.pixelSelection && !doc.pixelSelection.target ? doc.pixelSelection : null;
  const bounds = sel?.bounds ?? null;
  const key = JSON.stringify([doc?.documentId, doc?.version, paths, s, what]);
  const pages = result?.key === key ? result.pages : null;

  useEffect(() => {
    if (!doc && !paths) return;
    let alive = true;
    const timer = setTimeout(() => {
      run<{ pages: PrintPage[] }>("document.print_pages", {
        ...(paths ? { paths } : { documentId: doc?.documentId }),
        setup: s,
        ...(what === "selection" && bounds ? { rect: bounds } : {}),
      })
        .then((r) => alive && setResult({ key, pages: r.pages }))
        .catch((e) => useEditor.getState().showError(e));
    }, 200);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [doc, paths, s, what, bounds, key]);

  const select = <T extends string>(
    id: string,
    label: string,
    value: T,
    values: readonly T[],
    on: (v: T) => void,
  ) => (
    <div className="prop-field">
      <label htmlFor={id}>{label}</label>
      <select id={id} value={value} onChange={(e) => on(e.target.value as T)}>
        {values.map((v) => (
          <option key={v} value={v}>
            {tt(`${id.replace("print-", "")}_${v}`)}
          </option>
        ))}
      </select>
    </div>
  );

  return (
    <Dialog
      title={tt("title")}
      onCancel={close}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={!pages || printing}
            onClick={() => {
              if (!doc && !paths) return;
              remember(s);
              setPrinting(true);
              // EDT-114: a PDF of these exact pages, printed from the system's viewer.
              run("document.print", {
                ...(paths ? { paths } : { documentId: doc?.documentId }),
                setup: s,
                ...(what === "selection" && bounds ? { rect: bounds } : {}),
              })
                .then(() => {
                  useEditor.getState().showToast(tt("opened"));
                  close();
                })
                .catch((e) => useEditor.getState().showError(e))
                .finally(() => setPrinting(false));
            }}
          >
            {tt("print")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="print-layout">
        <div className="image-form">
          {select(
            "print-paper",
            tt("paper"),
            s.paper,
            ["a4", "letter", "legal", "a3"] as const,
            (paper) => set({ paper }),
          )}
          <div className="prop-field">
            <label htmlFor="print-orientation">{tt("orientation")}</label>
            <select
              id="print-orientation"
              value={s.orientation}
              onChange={(e) => set({ orientation: e.target.value as Setup["orientation"] })}
            >
              {(["auto", "portrait", "landscape"] as const).map((v) => (
                <option key={v} value={v}>
                  {t(`ui.editor.export.orient_${v}`)}
                </option>
              ))}
            </select>
          </div>
          <div className="prop-field">
            <label htmlFor="print-margin">{tt("margin")}</label>
            <span className="image-number">
              <input
                id="print-margin"
                type="number"
                min={0}
                max={50}
                value={s.margin}
                onChange={(e) =>
                  set({ margin: Math.max(0, Math.min(50, Number(e.target.value) || 0)) })
                }
              />
              <span className="muted">mm</span>
            </span>
          </div>
          {select("print-size", tt("size"), s.size, ["fit", "actual", "percent"] as const, (size) =>
            set({ size }),
          )}
          {s.size === "percent" && (
            <div className="prop-field">
              <label htmlFor="print-percent">{tt("percent")}</label>
              <span className="image-number">
                <input
                  id="print-percent"
                  type="number"
                  min={1}
                  max={1000}
                  value={s.percent}
                  onChange={(e) =>
                    set({ percent: Math.max(1, Math.min(1000, Number(e.target.value) || 100)) })
                  }
                />
                <span className="muted">%</span>
              </span>
            </div>
          )}
          {select(
            "print-position",
            tt("position"),
            s.position,
            ["center", "topLeft"] as const,
            (position) => set({ position }),
          )}
          {sel && (
            <div className="prop-field">
              <label htmlFor="print-what">{tt("what")}</label>
              <select
                id="print-what"
                value={what}
                onChange={(e) => setWhat(e.target.value as "all" | "selection")}
              >
                <option value="all">{tt("what_all")}</option>
                <option value="selection">{tt("what_selection")}</option>
              </select>
            </div>
          )}
        </div>
        <div className="print-preview" aria-label={tt("preview")}>
          {pages ? (
            <>
              <p className="muted">{tt("pages", { count: pages.length })}</p>
              <div className="print-thumbs">
                {pages.map((p, i) => (
                  <div
                    key={i}
                    className="print-thumb"
                    aria-label={tt("page_label", { n: i + 1 })}
                    style={{ aspectRatio: `${p.paper.w} / ${p.paper.h}` }}
                  >
                    <img
                      src={`data:image/png;base64,${p.png}`}
                      alt=""
                      style={{
                        left: `${(p.at.x / p.paper.w) * 100}%`,
                        top: `${(p.at.y / p.paper.h) * 100}%`,
                        width: `${(p.size.w / p.paper.w) * 100}%`,
                        height: `${(p.size.h / p.paper.h) * 100}%`,
                      }}
                    />
                  </div>
                ))}
              </div>
            </>
          ) : (
            <p className="muted">{tt("preparing")}</p>
          )}
        </div>
      </div>
    </Dialog>
  );
}
