// Import PDF (CAP-041): the pages as thumbnails, all checked; a range; the resolution,
// the result (one capture per page, or all in one image) and the page background. A
// PDF with a password asks for it first. The import runs in the core with a progress
// bar and Cancel; the window closes when it is done.
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { isCommandError, run } from "../lib/commands";
import { useAppShortcuts } from "../lib/appShortcuts";
import { formatRange, parseRange } from "./pdfRange";

interface Info {
  name: string;
  pages: number;
  needsPassword: boolean;
  wrongPassword?: boolean;
}

/** Most pages in one import (CAP-041). */
const MAX_PAGES = 200;
const DPIS = [72, 96, 150, 200, 300];

export function PdfImport({ id }: { id: string }) {
  const { t } = useTranslation();
  useAppShortcuts();
  const [info, setInfo] = useState<Info | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [asked, setAsked] = useState("");
  const [thumbs, setThumbs] = useState<Record<number, string>>({});
  const [checked, setChecked] = useState<number[]>([]);
  const [range, setRange] = useState("");
  const [rangeBad, setRangeBad] = useState(false);
  const [dpi, setDpi] = useState(150);
  const [combine, setCombine] = useState(false);
  const [transparent, setTransparent] = useState(false);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const alive = useRef(true);

  const close = useCallback(() => void getCurrentWebviewWindow().close(), []);

  const open = useCallback(
    (pw: string) => {
      run<Info>("capture.pdf_info", { id, password: pw })
        .then((i) => {
          setError(null);
          setInfo(i);
          if (!i.needsPassword) {
            setAsked(pw);
            const all = Array.from({ length: Math.min(i.pages, MAX_PAGES) }, (_, k) => k + 1);
            setChecked(all);
            setRange(formatRange(all));
          }
        })
        .catch((e: unknown) => setError(isCommandError(e) ? e.message : String(e)));
    },
    [id],
  );

  useEffect(() => {
    alive.current = true;
    open("");
    const un = listen<{ id: string; done: number; total: number }>("xs://pdf", (e) => {
      if (e.payload.id === id) setProgress({ done: e.payload.done, total: e.payload.total });
    });
    return () => {
      alive.current = false;
      void un.then((f) => f());
    };
  }, [id, open]);

  // Thumbnails, one after another, once the PDF is open.
  const pages = info && !info.needsPassword ? info.pages : 0;
  useEffect(() => {
    if (!pages) return;
    let stop = false;
    void (async () => {
      for (let page = 1; page <= Math.min(pages, MAX_PAGES) && !stop; page++) {
        try {
          const r = await run<{ png: string }>("capture.pdf_page", {
            id,
            page,
            width: 150,
            password: asked,
          });
          if (stop || !alive.current) return;
          setThumbs((m) => ({ ...m, [page]: `data:image/png;base64,${r.png}` }));
        } catch {
          return;
        }
      }
    })();
    return () => {
      stop = true;
    };
  }, [pages, id, asked]);

  const toggle = (page: number) => {
    const next = checked.includes(page) ? checked.filter((p) => p !== page) : [...checked, page];
    setChecked(next);
    setRange(formatRange(next));
    setRangeBad(false);
  };

  const commitRange = () => {
    const parsed = parseRange(range, pages);
    setRangeBad(parsed === null);
    if (parsed) setChecked(parsed);
  };

  const importNow = async () => {
    if (!checked.length) return;
    setError(null);
    setProgress({ done: 0, total: checked.length });
    try {
      await run("capture.pdf", {
        id,
        password: asked,
        pages: formatRange(checked),
        dpi,
        combine,
        background: transparent ? "transparent" : "white",
        dialog: false,
      });
      close();
    } catch (e) {
      setProgress(null);
      if (isCommandError(e) && e.code === "cancelled") return;
      setError(isCommandError(e) ? e.message : String(e));
    }
  };

  if (!info) {
    return (
      <div className="pdf-import">
        <p className="muted">{error ?? t("ui.pdf.loading")}</p>
      </div>
    );
  }

  if (info.needsPassword) {
    return (
      <form
        className="pdf-import pdf-password"
        onSubmit={(e) => {
          e.preventDefault();
          open(password);
        }}
      >
        <h1>{info.name}</h1>
        <p>{info.wrongPassword ? t("ui.pdf.password_wrong") : t("ui.pdf.password_needed")}</p>
        <label className="panel-field">
          <span>{t("ui.pdf.password")}</span>
          <input
            type="password"
            autoFocus
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </label>
        <div className="pdf-actions">
          <button type="submit" className="btn primary" disabled={!password}>
            {t("ui.pdf.open")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.pdf.cancel")}
          </button>
        </div>
      </form>
    );
  }

  const shown = Math.min(pages, MAX_PAGES);
  return (
    <div className="pdf-import">
      <h1>{info.name}</h1>
      <div className="pdf-body">
        <ul className="pdf-pages" aria-label={t("ui.pdf.pages")}>
          {Array.from({ length: shown }, (_, k) => k + 1).map((page) => {
            const on = checked.includes(page);
            return (
              <li key={page}>
                <label className={`pdf-page${on ? " on" : ""}`}>
                  <input type="checkbox" checked={on} onChange={() => toggle(page)} />
                  {thumbs[page] ? (
                    <img src={thumbs[page]} alt="" />
                  ) : (
                    <span className="pdf-thumb-wait" aria-hidden="true" />
                  )}
                  <span>{t("ui.pdf.page", { n: page })}</span>
                </label>
              </li>
            );
          })}
        </ul>
        <div className="pdf-options">
          <label className="panel-field">
            <span>{t("ui.pdf.range")}</span>
            <input
              value={range}
              placeholder={t("ui.pdf.range_hint")}
              aria-invalid={rangeBad}
              onChange={(e) => setRange(e.target.value)}
              onBlur={commitRange}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitRange();
              }}
            />
          </label>
          <div className="row">
            <button
              type="button"
              className="btn small"
              onClick={() => {
                const all = Array.from({ length: shown }, (_, k) => k + 1);
                setChecked(all);
                setRange(formatRange(all));
              }}
            >
              {t("ui.pdf.all")}
            </button>
            <button
              type="button"
              className="btn small"
              onClick={() => {
                setChecked([]);
                setRange("");
              }}
            >
              {t("ui.pdf.none")}
            </button>
          </div>
          <p className="muted" role="status">
            {t("ui.pdf.selected", { count: checked.length, total: pages })}
            {pages > MAX_PAGES && ` · ${t("ui.pdf.too_many")}`}
          </p>
          <label className="panel-field">
            <span>{t("ui.pdf.resolution")}</span>
            <select value={dpi} onChange={(e) => setDpi(Number(e.target.value))}>
              {DPIS.map((d) => (
                <option key={d} value={d}>
                  {d} ppp
                </option>
              ))}
            </select>
          </label>
          <label className="panel-field">
            <span>{t("ui.pdf.result")}</span>
            <select
              value={combine ? "one" : "pages"}
              onChange={(e) => setCombine(e.target.value === "one")}
            >
              <option value="pages">{t("ui.pdf.one_per_page")}</option>
              <option value="one">{t("ui.pdf.one_image")}</option>
            </select>
          </label>
          <label className="panel-field">
            <span>{t("ui.pdf.background")}</span>
            <select
              value={transparent ? "transparent" : "white"}
              onChange={(e) => setTransparent(e.target.value === "transparent")}
            >
              <option value="white">{t("ui.pdf.white")}</option>
              <option value="transparent">{t("ui.pdf.transparent")}</option>
            </select>
          </label>
        </div>
      </div>
      {error && (
        <p className="panel-status" role="alert">
          {error}
        </p>
      )}
      {progress && (
        <div className="pdf-progress" role="status">
          <progress value={progress.done} max={progress.total} />
          <span>
            {t("ui.pdf.importing", {
              done: Math.min(progress.done + 1, progress.total),
              total: progress.total,
            })}
          </span>
        </div>
      )}
      <div className="pdf-actions">
        <button
          type="button"
          className="btn primary"
          disabled={!checked.length || progress !== null}
          onClick={() => void importNow()}
        >
          {t("ui.pdf.import")}
        </button>
        <button
          type="button"
          className="btn"
          onClick={() =>
            progress ? void run("capture.pdf_cancel", { id }).catch(() => undefined) : close()
          }
        >
          {t("ui.pdf.cancel")}
        </button>
      </div>
    </div>
  );
}
