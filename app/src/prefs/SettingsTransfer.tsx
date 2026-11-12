// Preferences > Advanced > Export settings… and Import settings… (APP-011): an
// .xsettings file with the chosen groups; importing shows first what each group
// changes, and lets groups be left out.
import { open, save } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { changesOf, GROUPS, type Group } from "./transfer";

interface Preview {
  samePlatform: boolean;
  platform: string;
  groups: Partial<Record<Group, Record<string, unknown>>>;
}

function GroupChecks(props: {
  groups: Group[];
  chosen: Group[];
  set: (g: Group[]) => void;
  note?: (g: Group) => string | null;
}) {
  const { t } = useTranslation();
  return (
    <fieldset className="export-scope">
      <legend className="sr-only">{t("ui.prefs.transfer.groups")}</legend>
      {props.groups.map((g) => (
        <label key={g} className="panel-check">
          <input
            type="checkbox"
            checked={props.chosen.includes(g)}
            onChange={(e) =>
              props.set(
                e.target.checked ? [...props.chosen, g] : props.chosen.filter((x) => x !== g),
              )
            }
          />
          <span>
            {t(`ui.prefs.transfer.group_${g}`)}
            {props.note?.(g) && <span className="muted"> · {props.note(g)}</span>}
          </span>
        </label>
      ))}
    </fieldset>
  );
}

export function SettingsTransfer() {
  const { t } = useTranslation();
  const tt = (k: string, o?: Record<string, unknown>) => t(`ui.prefs.transfer.${k}`, o ?? {});
  const [exporting, setExporting] = useState(false);
  const [chosen, setChosen] = useState<Group[]>([...GROUPS]);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");

  const doExport = async () => {
    const path = await save({
      defaultPath: "XcreenShot.xsettings",
      filters: [{ name: tt("file_type"), extensions: ["xsettings"] }],
    });
    if (!path) return;
    try {
      await run("settings.export", { out: path, groups: chosen });
      setExporting(false);
      setStatus(tt("exported"));
    } catch (e) {
      setError(String((e as { message?: string }).message ?? e));
    }
  };

  return (
    <>
      <div className="setting">
        <span className="label">{tt("title")}</span>
        <div className="control">
          <button
            type="button"
            className="btn"
            onClick={() => {
              setChosen([...GROUPS]);
              setExporting(true);
            }}
          >
            {tt("export")}
          </button>
          <ImportSettingsButton
            onStatus={(text) => {
              setError("");
              setStatus(text);
            }}
            onError={setError}
          />
        </div>
        <div className="description" role="status">
          {error ? <span className="warning-text">{error}</span> : status || tt("help")}
        </div>
      </div>
      {exporting && (
        <Dialog
          title={tt("export_title")}
          onCancel={() => setExporting(false)}
          actions={
            <>
              <button
                type="button"
                className="btn primary"
                disabled={chosen.length === 0}
                onClick={() => void doExport()}
              >
                {tt("export_go")}
              </button>
              <button type="button" className="btn" onClick={() => setExporting(false)}>
                {t("ui.editor.cancel")}
              </button>
            </>
          }
        >
          <p className="muted">{tt("export_help")}</p>
          <GroupChecks groups={[...GROUPS]} chosen={chosen} set={setChosen} />
        </Dialog>
      )}
    </>
  );
}

/** Import settings… (APP-011): choose an .xsettings file, see what each group changes,
 * import. Also in the welcome, for a new computer (APP-055). */
export function ImportSettingsButton(props: {
  onStatus: (text: string) => void;
  onError: (text: string) => void;
}) {
  const { t } = useTranslation();
  const tt = (k: string) => t(`ui.prefs.transfer.${k}`);
  const [chosen, setChosen] = useState<Group[]>([...GROUPS]);
  const [importing, setImporting] = useState<{ path: string; preview: Preview } | null>(null);
  const pick = async () => {
    props.onError("");
    const path = await open({ filters: [{ name: tt("file_type"), extensions: ["xsettings"] }] });
    if (typeof path !== "string") return;
    try {
      const preview = await run<Preview>("settings.import", { path, dryRun: true });
      setChosen(GROUPS.filter((g) => preview.groups[g] !== undefined));
      setImporting({ path, preview });
    } catch (e) {
      props.onError(String((e as { message?: string }).message ?? e));
    }
  };
  return (
    <>
      <button type="button" className="btn" onClick={() => void pick()}>
        {tt("import")}
      </button>
      {importing && (
        <ImportDialog
          path={importing.path}
          preview={importing.preview}
          chosen={chosen}
          setChosen={setChosen}
          done={(text) => {
            setImporting(null);
            props.onStatus(text);
          }}
          cancel={() => setImporting(null)}
        />
      )}
    </>
  );
}

function ImportDialog(props: {
  path: string;
  preview: Preview;
  chosen: Group[];
  setChosen: (g: Group[]) => void;
  done: (text: string) => void;
  cancel: () => void;
}) {
  const { t } = useTranslation();
  const tt = (k: string, o?: Record<string, unknown>) => t(`ui.prefs.transfer.${k}`, o ?? {});
  const { preview } = props;
  const [convert, setConvert] = useState(true);
  const [onConflict, setOnConflict] = useState<"keepBoth" | "replace" | "skip">("keepBoth");
  const [error, setError] = useState("");
  const present = GROUPS.filter((g) => preview.groups[g] !== undefined);
  const note = (g: Group) => tt("changes", { count: changesOf(g, preview.groups[g]) });
  const go = async () => {
    try {
      const r = await run<Preview>("settings.import", {
        path: props.path,
        groups: props.chosen,
        convertShortcuts: convert,
        onConflict,
      });
      const sc = r.groups.shortcuts as
        { unconvertible?: unknown[]; conflicts?: unknown[] } | undefined;
      const left = (sc?.unconvertible?.length ?? 0) + (sc?.conflicts?.length ?? 0);
      props.done(
        left > 0 ? `${tt("imported")} ${tt("unassigned", { count: left })}` : tt("imported"),
      );
    } catch (e) {
      setError(String((e as { message?: string }).message ?? e));
    }
  };
  return (
    <Dialog
      title={tt("import_title")}
      onCancel={props.cancel}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={props.chosen.length === 0}
            onClick={() => void go()}
          >
            {tt("import_go")}
          </button>
          <button type="button" className="btn" onClick={props.cancel}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p className="muted">{tt("import_help")}</p>
      <GroupChecks groups={present} chosen={props.chosen} set={props.setChosen} note={note} />
      {!preview.samePlatform && props.chosen.includes("shortcuts") && (
        <label className="panel-check">
          <input type="checkbox" checked={convert} onChange={(e) => setConvert(e.target.checked)} />
          {tt("convert", { platform: preview.platform === "macos" ? "macOS" : "Windows" })}
        </label>
      )}
      {props.chosen.includes("themes") && (
        <label className="prop-field">
          <span>{tt("conflict")}</span>
          <select
            value={onConflict}
            onChange={(e) => setOnConflict(e.target.value as typeof onConflict)}
          >
            <option value="keepBoth">{tt("conflict_both")}</option>
            <option value="replace">{tt("conflict_replace")}</option>
            <option value="skip">{tt("conflict_skip")}</option>
          </select>
        </label>
      )}
      {error && <p className="warning-text">{error}</p>}
    </Dialog>
  );
}
