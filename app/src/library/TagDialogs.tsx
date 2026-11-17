// The tag editor (LIB-009) and the dialogs of a tag in the filter pane (LIB-010):
// rename, merge, delete.
import { useEffect, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import {
  allTags,
  changeTags,
  deleteTag,
  mergeTag,
  renameTag,
  useTagAction,
  useTagEditor,
} from "./marks";
import { run } from "../lib/commands";
import { useLibrary, type LibraryItem } from "./state";

export function TagDialogs() {
  const editing = useTagEditor((s) => s.ids);
  const pending = useTagAction((s) => s.pending);
  return (
    <>
      {editing && <TagEditor ids={editing} />}
      {pending && <TagAction />}
    </>
  );
}

/** Tags of the selected items: common ones as chips, partial ones dotted with "in 3 of
 * 5"; adding applies to all, removing removes from all (LIB-009). */
function TagEditor({ ids }: { ids: string[] }) {
  const { t } = useTranslation();
  const all = useLibrary((s) => s.items);
  // Items the Library view has not loaded (the tray's, TRY-016) are read here, again
  // after each change.
  const [read, setRead] = useState<LibraryItem[]>([]);
  const [round, setRound] = useState(0);
  const missing = ids.filter((id) => !all.some((i) => i.id === id));
  const key = missing.join(",");
  useEffect(() => {
    if (!key) return;
    let alive = true;
    void Promise.all(
      key.split(",").map((id) => run<LibraryItem>("library.get", { id }).catch(() => null)),
    ).then((list) => alive && setRead(list.filter((i): i is LibraryItem => i !== null)));
    return () => {
      alive = false;
    };
  }, [key, round]);
  const items = [
    ...all.filter((i) => ids.includes(i.id)),
    ...read.filter((i) => missing.includes(i.id)),
  ];
  const [draft, setDraft] = useState("");
  const [known, setKnown] = useState<string[]>([]);
  useEffect(() => {
    let alive = true;
    void allTags().then((list) => alive && setKnown(list.map((x) => x.tag)));
    return () => {
      alive = false;
    };
  }, []);
  const close = () => useTagEditor.setState({ ids: null });
  // Each tag once, whatever its case, with how many of the items have it.
  const counts: { tag: string; n: number }[] = [];
  for (const tag of items.flatMap((i) => i.tags)) {
    const c = counts.find((x) => x.tag.toLowerCase() === tag.toLowerCase());
    if (c) c.n += 1;
    else counts.push({ tag, n: 1 });
  }
  const total = items.length;
  const add = async (raw: string) => {
    const tags = raw
      .split(",")
      .map((x) => x.trim())
      .filter(Boolean);
    if (tags.length === 0) return;
    if (await changeTags(ids, tags, [])) setDraft("");
    setRound((n) => n + 1);
  };
  const remove = (tag: string) =>
    void changeTags(ids, [], [tag]).then(() => setRound((n) => n + 1));
  const onKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      void add(draft);
    } else if (e.key === "Backspace" && draft === "") {
      const last = counts.filter((c) => c.n === total).at(-1);
      if (last) remove(last.tag);
    }
  };
  const suggestions = known.filter(
    (k) => !counts.some((c) => c.n === total && c.tag.toLowerCase() === k.toLowerCase()),
  );
  return (
    <Dialog
      title={
        total === 1
          ? t("ui.library.tags.title_one", { name: items[0]?.name ?? "" })
          : t("ui.library.tags.title_many", { count: total })
      }
      onCancel={close}
      actions={
        <button type="button" className="btn primary" onClick={close}>
          {t("ui.library.tags.done")}
        </button>
      }
    >
      <div className="tag-chips" role="list" aria-label={t("ui.library.filter.tags")}>
        {counts.length === 0 && <span className="muted">{t("ui.library.tags.none")}</span>}
        {counts.map((c) => {
          const partial = c.n < total;
          const label = partial
            ? t("ui.library.tags.partial", { tag: c.tag, count: c.n, total })
            : c.tag;
          return (
            <span
              key={c.tag}
              role="listitem"
              className={`tag-chip${partial ? " partial" : ""}`}
              tabIndex={0}
              title={partial ? t("ui.library.tags.add_to_all") : undefined}
              onClick={() => partial && void changeTags(ids, [c.tag], [])}
              onKeyDown={(e) => {
                if (e.key === "Delete" || e.key === "Backspace") {
                  e.preventDefault();
                  remove(c.tag);
                }
              }}
            >
              {label}
              <button
                type="button"
                className="tag-x"
                aria-label={t("ui.library.tags.remove", { tag: c.tag })}
                onClick={(e) => {
                  e.stopPropagation();
                  remove(c.tag);
                }}
              >
                ✕
              </button>
            </span>
          );
        })}
      </div>
      <div className="prop-field">
        <label htmlFor="tag-input">{t("ui.library.tags.add")}</label>
        <input
          id="tag-input"
          data-autofocus
          list="tag-known"
          value={draft}
          maxLength={50}
          placeholder={t("ui.library.tags.placeholder")}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={onKey}
        />
        <datalist id="tag-known">
          {suggestions.map((k) => (
            <option key={k} value={k} />
          ))}
        </datalist>
      </div>
    </Dialog>
  );
}

/** Rename, merge or delete a tag on all its items (LIB-010). */
function TagAction() {
  const { t } = useTranslation();
  const pending = useTagAction((s) => s.pending);
  const tags = useLibrary((s) => s.facets.tags);
  const [name, setName] = useState(pending?.tag ?? "");
  const [into, setInto] = useState("");
  const [exists, setExists] = useState<string | null>(null);
  if (!pending) return null;
  const close = () => useTagAction.setState({ pending: null });
  const others = tags
    .map((c) => c.value)
    .filter((v) => v.toLowerCase() !== pending.tag.toLowerCase());
  if (pending.action === "delete")
    return (
      <Dialog
        title={t("ui.library.tags.delete_title", { tag: pending.tag })}
        onCancel={close}
        actions={
          <>
            <button type="button" className="btn" onClick={close}>
              {t("ui.editor.cancel")}
            </button>
            <button
              type="button"
              className="btn danger"
              onClick={() => {
                close();
                void deleteTag(pending.tag);
              }}
            >
              {t("ui.library.tags.delete")}
            </button>
          </>
        }
      >
        <p>{t("ui.library.tags.delete_body", { tag: pending.tag, count: pending.count })}</p>
      </Dialog>
    );
  if (pending.action === "merge")
    return (
      <Dialog
        title={t("ui.library.tags.merge_title", { tag: pending.tag })}
        onCancel={close}
        actions={
          <>
            <button type="button" className="btn" onClick={close}>
              {t("ui.editor.cancel")}
            </button>
            <button
              type="button"
              className="btn primary"
              disabled={!into}
              onClick={() => {
                close();
                void mergeTag(pending.tag, into);
              }}
            >
              {t("ui.library.tags.merge")}
            </button>
          </>
        }
      >
        <div className="prop-field">
          <label htmlFor="tag-into">{t("ui.library.tags.merge_into")}</label>
          <select
            id="tag-into"
            data-autofocus
            value={into}
            onChange={(e) => setInto(e.target.value)}
          >
            <option value="" />
            {others.map((o) => (
              <option key={o} value={o}>
                {o}
              </option>
            ))}
          </select>
        </div>
      </Dialog>
    );
  const rename = async (merge: boolean) => {
    const r = await renameTag(pending.tag, name.trim(), merge);
    if (r === "exists") setExists(name.trim());
    else if (r === "ok") close();
  };
  return (
    <Dialog
      title={t("ui.library.tags.rename_title", { tag: pending.tag })}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
          {exists ? (
            <button type="button" className="btn primary" onClick={() => void rename(true)}>
              {t("ui.library.tags.merge")}
            </button>
          ) : (
            <button
              type="button"
              className="btn primary"
              disabled={!name.trim()}
              onClick={() => void rename(false)}
            >
              {t("ui.library.tags.rename")}
            </button>
          )}
        </>
      }
    >
      <div className="prop-field">
        <label htmlFor="tag-name">{t("ui.library.tags.new_name")}</label>
        <input
          id="tag-name"
          data-autofocus
          value={name}
          maxLength={50}
          onChange={(e) => {
            setName(e.target.value);
            setExists(null);
          }}
          onKeyDown={(e) => e.key === "Enter" && name.trim() && void rename(Boolean(exists))}
        />
      </div>
      {exists && (
        <p className="tpl-warning" role="alert">
          {t("ui.library.tags.exists", { tag: exists })}
        </p>
      )}
    </Dialog>
  );
}
