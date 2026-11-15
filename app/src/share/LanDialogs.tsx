// The local network's dialogs in the main window (SHR-043, SHR-044): someone wants to
// pair, someone sends something, and Send to contacts.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { useEditor } from "../editor/state";
import {
  answerIncoming,
  answerPairing,
  fileSize,
  followLan,
  sendTo,
  useLan,
  type LanPairing,
} from "./lanState";

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** The code both people compare (SHR-043). */
export function PairCode({ pairing }: { pairing: LanPairing }) {
  const { t } = useTranslation();
  return (
    <div className="lan-code" aria-label={t("ui.lan.code_label", { code: pairing.code })}>
      {pairing.code.slice(0, 3)} {pairing.code.slice(3)}
    </div>
  );
}

function PairDialog({ pairing }: { pairing: LanPairing }) {
  const { t } = useTranslation();
  const answer = (accept: boolean) => void answerPairing(pairing.id, accept).catch(fail);
  return (
    <Dialog
      title={t(pairing.direction === "in" ? "ui.lan.pair_in_title" : "ui.lan.pair_out_title")}
      onCancel={() => answer(false)}
      actions={
        <>
          <button type="button" className="btn" onClick={() => answer(false)}>
            {t(pairing.direction === "in" ? "ui.lan.decline" : "ui.lan.codes_differ")}
          </button>
          <button type="button" className="btn primary" onClick={() => answer(true)}>
            {t("ui.lan.codes_match")}
          </button>
        </>
      }
    >
      <p>
        {t(pairing.direction === "in" ? "ui.lan.pair_in" : "ui.lan.pair_out", {
          name: pairing.name,
          device: pairing.device,
        })}
      </p>
      <PairCode pairing={pairing} />
      <p className="muted">{t("ui.lan.pair_check")}</p>
    </Dialog>
  );
}

function IncomingDialog() {
  const { t, i18n } = useTranslation();
  const offer = useLan((s) => s.incoming.find((i) => !i.receiving));
  if (!offer) return null;
  const answer = (accept: boolean) => void answerIncoming(offer.id, accept).catch(fail);
  return (
    <Dialog
      title={t("ui.lan.incoming_title")}
      onCancel={() => answer(false)}
      actions={
        <>
          <button type="button" className="btn" onClick={() => answer(false)}>
            {t("ui.lan.reject")}
          </button>
          <button type="button" className="btn primary" onClick={() => answer(true)}>
            {t("ui.lan.accept")}
          </button>
        </>
      }
    >
      <p>
        {t("ui.lan.incoming", {
          name: offer.name,
          count: offer.items.length,
          size: fileSize(offer.total, i18n.language),
        })}
      </p>
      <ul className="lan-files">
        {offer.items.map((f) => (
          <li key={f.name}>
            {f.name} · {fileSize(f.size, i18n.language)}
          </li>
        ))}
      </ul>
      <p className="muted">{t("ui.lan.incoming_where")}</p>
    </Dialog>
  );
}

/** Send to contacts (SHR-044). */
function SendDialog({ paths }: { paths: string[] }) {
  const { t } = useTranslation();
  // The list itself, filtered here: a new array from the selector would redraw forever.
  const people = useLan((s) => s.people);
  const contacts = people.filter((p) => p.contact);
  const [chosen, setChosen] = useState<string[]>([]);
  const close = () => useLan.setState({ sending: null });
  return (
    <Dialog
      title={t("ui.lan.send_title", { count: paths.length })}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={chosen.length === 0}
            onClick={() =>
              void sendTo(chosen, paths)
                .then(() => {
                  useEditor.getState().showToast(t("ui.lan.sent", { count: chosen.length }));
                  close();
                })
                .catch(fail)
            }
          >
            {t("ui.lan.send")}
          </button>
        </>
      }
    >
      {contacts.length === 0 ? (
        <p>{t("ui.lan.no_contacts")}</p>
      ) : (
        <fieldset className="lan-choose">
          <legend>{t("ui.lan.contacts")}</legend>
          {contacts.map((c) => (
            <label key={c.fingerprint} className="panel-check">
              <input
                type="checkbox"
                checked={chosen.includes(c.fingerprint)}
                onChange={(e) =>
                  setChosen((list) =>
                    e.target.checked
                      ? [...list, c.fingerprint]
                      : list.filter((x) => x !== c.fingerprint),
                  )
                }
              />
              {c.name} · {c.device}{" "}
              <span className={c.online ? "lan-online" : "muted"}>
                {t(c.online ? "ui.lan.online" : "ui.lan.offline")}
              </span>
            </label>
          ))}
        </fieldset>
      )}
      <p className="muted">{t("ui.lan.send_later")}</p>
    </Dialog>
  );
}

export function LanDialogs() {
  useEffect(() => followLan(), []);
  const pairing = useLan((s) => s.pairings.find((p) => !p.answered));
  const sending = useLan((s) => s.sending);
  return (
    <>
      {pairing && <PairDialog key={pairing.id} pairing={pairing} />}
      {!pairing && <IncomingDialog />}
      {sending && <SendDialog paths={sending} />}
    </>
  );
}
