// Preferences > Share > Local network (SHR-042 to SHR-044): this computer, the people on
// the network, the contacts and the sends.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { PairCode } from "../share/LanDialogs";
import {
  answerPairing,
  cancelLan,
  contactAction,
  fileSize,
  followLan,
  lanFail,
  pair,
  useLan,
  type LanPerson,
} from "../share/lanState";

const fail = lanFail;

function when(iso: string | null | undefined, lang: string): string {
  if (!iso) return "";
  return new Date(iso).toLocaleString(lang, { dateStyle: "medium", timeStyle: "short" });
}

function Contact({ c }: { c: LanPerson }) {
  const { t, i18n } = useTranslation();
  const [alias, setAlias] = useState(c.name);
  return (
    <li className="lan-row">
      <div className="lan-who">
        <input
          aria-label={t("ui.lan.contact_name")}
          value={alias}
          onChange={(e) => setAlias(e.target.value)}
          onBlur={() => {
            if (alias.trim() !== c.name)
              void contactAction(c.fingerprint, "rename", alias.trim()).catch(fail);
          }}
        />
        <span className="muted">
          {c.device} ·{" "}
          {c.online
            ? t("ui.lan.online")
            : t("ui.lan.last_seen", { when: when(c.lastSeen, i18n.language) || "—" })}
        </span>
      </div>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={!!c.alwaysAccept}
          onChange={(e) =>
            void contactAction(c.fingerprint, "alwaysAccept", e.target.checked).catch(fail)
          }
        />
        {t("ui.lan.always_accept")}
      </label>
      <button
        type="button"
        className="btn small"
        onClick={() => void contactAction(c.fingerprint, "remove").catch(fail)}
      >
        {t("ui.lan.remove")}
      </button>
      <button
        type="button"
        className="btn small"
        onClick={() => void contactAction(c.fingerprint, "block").catch(fail)}
      >
        {t("ui.lan.block")}
      </button>
    </li>
  );
}

export function LanSection() {
  const { t, i18n } = useTranslation();
  useEffect(() => followLan(), []);
  const status = useLan((s) => s.status);
  const people = useLan((s) => s.people);
  const pairings = useLan((s) => s.pairings);
  const sends = useLan((s) => s.sends);
  const progress = useLan((s) => s.progress);
  const error = useLan((s) => s.error);
  if (!status) return null;
  const contacts = people.filter((p) => p.contact);
  const others = people.filter((p) => !p.contact && !p.blocked);
  const blocked = people.filter((p) => p.blocked);
  return (
    <section className="lan-section" aria-label={t("ui.lan.title")}>
      <h3>{t("ui.lan.title")}</h3>
      {error && (
        <p className="lib-warning-text" role="alert">
          {error}{" "}
          <button
            type="button"
            className="btn small"
            onClick={() => useLan.setState({ error: null })}
          >
            {t("ui.editor.close")}
          </button>
        </p>
      )}
      {!status.enabled ? (
        <p className="muted">{t("ui.lan.firewall")}</p>
      ) : (
        <>
          <p className="muted">
            {status.running
              ? t("ui.lan.me", {
                  name: status.name,
                  device: status.device,
                  fingerprint: status.fingerprint ?? "",
                })
              : t("ui.lan.not_running", { reason: status.error ?? "" })}
          </p>
          {pairings
            .filter((p) => !p.answered)
            .map((p) => (
              <div
                key={p.id}
                className="lan-pairing"
                role="group"
                aria-label={t("ui.lan.pair_out_title")}
              >
                <p>
                  {t(p.direction === "in" ? "ui.lan.pair_in" : "ui.lan.pair_out", {
                    name: p.name,
                    device: p.device,
                  })}
                </p>
                <PairCode pairing={p} />
                <div className="actions">
                  <button
                    type="button"
                    className="btn"
                    onClick={() => void answerPairing(p.id, false).catch(fail)}
                  >
                    {t(p.direction === "in" ? "ui.lan.decline" : "ui.lan.codes_differ")}
                  </button>
                  <button
                    type="button"
                    className="btn primary"
                    onClick={() => void answerPairing(p.id, true).catch(fail)}
                  >
                    {t("ui.lan.codes_match")}
                  </button>
                </div>
              </div>
            ))}
          <h4>{t("ui.lan.people")}</h4>
          {others.length === 0 ? (
            <p className="muted">{t("ui.lan.nobody")}</p>
          ) : (
            <ul className="lan-list">
              {others.map((p) => (
                <li key={p.fingerprint} className="lan-row">
                  <div className="lan-who">
                    <strong>{p.name}</strong>
                    <span className="muted">{p.device}</span>
                  </div>
                  <button
                    type="button"
                    className="btn small"
                    onClick={() => void pair(p.fingerprint).catch(fail)}
                  >
                    {t("ui.lan.pair")}
                  </button>
                  <button
                    type="button"
                    className="btn small"
                    onClick={() => void contactAction(p.fingerprint, "block").catch(fail)}
                  >
                    {t("ui.lan.block")}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
      <h4>{t("ui.lan.contacts")}</h4>
      {contacts.length === 0 ? (
        <p className="muted">{t("ui.lan.no_contacts")}</p>
      ) : (
        <ul className="lan-list">
          {contacts.map((c) => (
            <Contact key={c.fingerprint} c={c} />
          ))}
        </ul>
      )}
      {blocked.length > 0 && (
        <>
          <h4>{t("ui.lan.blocked")}</h4>
          <ul className="lan-list">
            {blocked.map((p) => (
              <li key={p.fingerprint} className="lan-row">
                <div className="lan-who">
                  <strong>{p.name}</strong>
                  <span className="muted">{p.device}</span>
                </div>
                <button
                  type="button"
                  className="btn small"
                  onClick={() => void contactAction(p.fingerprint, "unblock").catch(fail)}
                >
                  {t("ui.lan.unblock")}
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
      <h4>{t("ui.lan.sends")}</h4>
      {sends.length === 0 ? (
        <p className="muted">{t("ui.lan.no_sends")}</p>
      ) : (
        <ul className="lan-list">
          {sends.slice(0, 50).map((s) => {
            const p = progress[s.id];
            return (
              <li key={s.id} className="lan-row">
                <div className="lan-who">
                  <strong>
                    {t("ui.lan.send_line", {
                      count: s.items.length,
                      name: s.toName || s.to,
                      size: fileSize(
                        s.items.reduce((n, i) => n + i.size, 0),
                        i18n.language,
                      ),
                    })}
                  </strong>
                  <span className="muted">
                    {t(`ui.lan.status.${s.status}`)}
                    {s.status === "sending" && p && p.total > 0
                      ? ` · ${Math.round((p.done / p.total) * 100)} %`
                      : ""}
                    {" · "}
                    {when(s.updated || s.created, i18n.language)}
                  </span>
                </div>
                {(s.status === "pending" || s.status === "sending") && (
                  <button
                    type="button"
                    className="btn small"
                    onClick={() => void cancelLan(s.id).catch(fail)}
                  >
                    {t("ui.editor.cancel")}
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
