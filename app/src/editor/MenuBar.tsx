// The editor's menu bar on Windows (EDT-002), drawn by the app so its menus spring in like
// every other menu (APP-012). Same entries and commands as the shell's menus
// (`xs_editor_menus`). `Alt` or `F10` reach it, `Alt`+letter opens a menu (its letter
// is underlined while the keyboard is in use), the arrows move through it. macOS keeps
// the system menu bar.
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { MenuList, type CloseReason } from "../components/Menu";
import { usePresence } from "../components/usePresence";
import { menuItems, mnemonic, useMenuState, type MenuNode } from "./menus";

type TopMenu = Extract<MenuNode, { kind: "submenu" }>;

/** A label with its access key underlined (shown only in keyboard use, by CSS). */
function Label({ text }: { text: string }) {
  const { text: plain, at } = mnemonic(text);
  if (at < 0) return <>{plain}</>;
  return (
    <>
      {plain.slice(0, at)}
      <span className="mnemonic">{plain.charAt(at)}</span>
      {plain.slice(at + 1)}
    </>
  );
}

export function MenuBar() {
  const { t } = useTranslation();
  const state = useMenuState();
  const [menus, setMenus] = useState<TopMenu[]>([]);
  const [open, setOpen] = useState<number | null>(null);
  const [shown, setShown] = useState(0);
  const [focus, setFocus] = useState<"first" | "menu">("menu");
  const [keys, setKeys] = useState(false);
  const presence = usePresence(open !== null);
  const bar = useRef<HTMLDivElement>(null);
  const tops = useRef<(HTMLButtonElement | null)[]>([]);
  const list = useRef<HTMLDivElement>(null);
  /** Where focus was before the keyboard came to the bar; it goes back there. */
  const before = useRef<HTMLElement | null>(null);
  // The handlers below are set once; they read the menus through this ref.
  const keysOf = useRef<(string | null)[]>([]);
  useEffect(() => {
    keysOf.current = menus.map((m) => mnemonic(t(m.label)).key);
  }, [menus, t]);

  useEffect(() => {
    invoke<MenuNode[]>("xs_editor_menus")
      .then((all) => setMenus(all.filter((m): m is TopMenu => m.kind === "submenu")))
      .catch(() => undefined);
  }, []);

  const show = (i: number, how: "first" | "menu") => {
    setShown(i);
    setFocus(how);
    setOpen(i);
  };
  const remember = () => {
    if (bar.current?.contains(document.activeElement)) return;
    before.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  };
  const leave = () => {
    setOpen(null);
    setKeys(false);
    const back = before.current;
    before.current = null;
    if (back?.isConnected) back.focus({ preventScroll: true });
    else if (bar.current?.contains(document.activeElement))
      (document.activeElement as HTMLElement).blur();
  };
  const actions = useRef({ show, remember, leave });
  useEffect(() => {
    actions.current = { show, remember, leave };
  });

  // Alt pressed and released alone, or F10, moves focus to the bar and back; Alt plus a
  // menu's letter opens it (EDT-002).
  useEffect(() => {
    let alone = false;
    const toggle = () => {
      const a = actions.current;
      if (bar.current?.contains(document.activeElement)) {
        a.leave();
        return;
      }
      a.remember();
      setKeys(true);
      tops.current[0]?.focus({ preventScroll: true });
    };
    const down = (e: KeyboardEvent) => {
      if (e.key === "Alt") {
        alone = !e.repeat;
        if (alone) e.preventDefault();
        return;
      }
      alone = false;
      if (e.key === "F10" && !e.shiftKey && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        toggle();
        return;
      }
      if (e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey && e.key.length === 1) {
        const i = keysOf.current.indexOf(e.key.toLowerCase());
        if (i < 0) return;
        e.preventDefault();
        actions.current.remember();
        setKeys(true);
        actions.current.show(i, "first");
      }
    };
    const up = (e: KeyboardEvent) => {
      if (e.key === "Alt" && alone) {
        e.preventDefault();
        alone = false;
        toggle();
      }
    };
    const pointer = () => {
      alone = false;
      setKeys(false);
    };
    const reset = () => {
      alone = false;
    };
    window.addEventListener("keydown", down, true);
    window.addEventListener("keyup", up, true);
    window.addEventListener("pointerdown", pointer, true);
    window.addEventListener("blur", reset);
    return () => {
      window.removeEventListener("keydown", down, true);
      window.removeEventListener("keyup", up, true);
      window.removeEventListener("pointerdown", pointer, true);
      window.removeEventListener("blur", reset);
    };
  }, []);

  // A click outside closes the open menu.
  useEffect(() => {
    if (open === null) return;
    const onDown = (e: PointerEvent) => {
      if (!bar.current?.contains(e.target as Node)) setOpen(null);
    };
    document.addEventListener("pointerdown", onDown, true);
    return () => document.removeEventListener("pointerdown", onDown, true);
  }, [open]);

  const onClose = (reason: CloseReason) => {
    if (reason === "escape") {
      setOpen(null);
      tops.current[shown]?.focus({ preventScroll: true });
    } else if (reason === "select") {
      leave();
    } else {
      setOpen(null);
    }
  };
  const step = (from: number, direction: -1 | 1) =>
    (from + direction + menus.length) % menus.length;

  const onTopKey = (e: ReactKeyboardEvent<HTMLButtonElement>, i: number) => {
    switch (e.key) {
      case "ArrowRight":
      case "ArrowLeft": {
        e.preventDefault();
        const next = step(i, e.key === "ArrowRight" ? 1 : -1);
        if (open !== null) show(next, "first");
        else tops.current[next]?.focus({ preventScroll: true });
        break;
      }
      case "Home":
      case "End":
        e.preventDefault();
        tops.current[e.key === "Home" ? 0 : menus.length - 1]?.focus({ preventScroll: true });
        break;
      case "ArrowDown":
      case "Enter":
      case " ":
        e.preventDefault();
        show(i, "first");
        break;
      case "Escape":
        e.preventDefault();
        leave();
        break;
    }
  };

  const current = menus[shown];
  return (
    <div
      className="menubar"
      role="menubar"
      aria-label={t("ui.editor.menubar")}
      ref={bar}
      data-keys={keys || undefined}
    >
      {menus.map((menu, i) => {
        const label = t(menu.label);
        const key = mnemonic(label).key;
        return (
          <div className="menubar-item" key={menu.label}>
            <button
              ref={(el) => {
                tops.current[i] = el;
              }}
              type="button"
              role="menuitem"
              tabIndex={-1}
              className="menubar-top"
              aria-haspopup="menu"
              aria-expanded={open === i}
              aria-keyshortcuts={key ? `Alt+${key.toUpperCase()}` : undefined}
              onClick={() => (open === i ? setOpen(null) : show(i, "menu"))}
              onPointerEnter={() => {
                if (open !== null && open !== i) show(i, "menu");
              }}
              onKeyDown={(e) => onTopKey(e, i)}
            >
              <Label text={label} />
            </button>
            {presence.mounted && shown === i && current && (
              <MenuList
                key={i}
                menuRef={list}
                label={mnemonic(label).text}
                items={menuItems(current.items, t, state)}
                state={presence.state}
                focus={focus}
                className="menu"
                onClose={onClose}
                onExited={presence.exited}
                onSide={(direction) => show(step(i, direction), "first")}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}
