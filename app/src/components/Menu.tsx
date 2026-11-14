// Animated menus: a dropdown (Menu) and a pointer menu (ContextMenu) that share one list.
// They spring in from their anchor corner, stagger their items and fade out before
// unmounting; with reduced motion they only fade (APP-045). Keyboard per APP-043.
import {
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
  type RefObject,
} from "react";
import { createPortal } from "react-dom";
import { Icon } from "./Icon";
import { usePresence } from "./usePresence";

export interface MenuItem {
  id: string;
  label: ReactNode;
  /** Set (true or false) for a choice that shows a check mark when chosen. */
  checked?: boolean | undefined;
  disabled?: boolean;
  /** Tooltip: why it is off, for instance. */
  title?: string;
  /** Extra content after the label, such as a setting key. */
  trailing?: ReactNode;
  /** Shortcut shown at the right, already written for this platform. */
  shortcut?: string | undefined;
  /** Draws a separator above the item. */
  separator?: boolean;
  /** Entries of a submenu that opens beside the item (EDT-002: "Toolbar ▸"). */
  submenu?: MenuItem[];
  onSelect: () => void;
}

export type CloseReason = "select" | "escape" | "tab" | "outside" | "blur";

/** A submenu follows the pointer after this pause, so crossing an item does not open it. */
const SUBMENU_DELAY_MS = 120;

/** The menu's own items: a submenu's items belong to the submenu. */
function enabledItems(menu: HTMLElement): HTMLElement[] {
  return Array.from(
    menu.querySelectorAll<HTMLElement>(':scope > [role^="menuitem"]:not([aria-disabled="true"])'),
  );
}

export interface MenuListProps {
  items: MenuItem[];
  state: "open" | "closed";
  /** Where focus goes when the menu opens. */
  focus: "first" | "menu";
  className: string;
  menuRef: RefObject<HTMLDivElement>;
  id?: string;
  label?: string | undefined;
  labelledBy?: string;
  style?: CSSProperties;
  onClose: (reason: CloseReason) => void;
  onExited: () => void;
  /** Left or Right that the list does not use itself: the menu bar moves to the next
   * menu, a submenu goes back to its parent. */
  onSide?: (direction: -1 | 1) => void;
}

export function MenuList({
  items,
  state,
  focus,
  className,
  menuRef,
  id,
  label,
  labelledBy,
  style,
  onClose,
  onExited,
  onSide,
}: MenuListProps) {
  const [sub, setSub] = useState<{ id: string; focus: "first" | "menu"; top: number } | null>(null);
  const subMenu = useRef<HTMLDivElement>(null);
  const hover = useRef<number>();
  useEffect(() => () => window.clearTimeout(hover.current), []);

  useEffect(() => {
    const menu = menuRef.current;
    if (state !== "open" || !menu) return;
    if (focus === "first") enabledItems(menu)[0]?.focus({ preventScroll: true });
    else menu.focus({ preventScroll: true });
  }, [state, focus, menuRef]);

  const opener = (itemId: string) =>
    Array.from(menuRef.current?.children ?? []).find(
      (el): el is HTMLElement => el instanceof HTMLElement && el.dataset.item === itemId,
    );
  // Measured when it opens (an event), beside its item.
  const openSub = (itemId: string, how: "first" | "menu") =>
    setSub({ id: itemId, focus: how, top: (opener(itemId)?.offsetTop ?? 0) - 7 });
  const closeSub = () => {
    if (!sub) return;
    opener(sub.id)?.focus({ preventScroll: true });
    setSub(null);
  };

  const onKeyDown = (e: ReactKeyboardEvent<HTMLDivElement>) => {
    const list = enabledItems(e.currentTarget);
    // Keys pressed inside an open submenu are the submenu's.
    if (e.target !== e.currentTarget && !list.includes(e.target as HTMLElement)) return;
    const at = list.indexOf(document.activeElement as HTMLElement);
    const current = at >= 0 ? items.find((it) => it.id === list[at]?.dataset.item) : undefined;
    const go = (n: number) => {
      e.preventDefault();
      e.stopPropagation();
      list[(n + list.length) % list.length]?.focus();
    };
    switch (e.key) {
      case "ArrowDown":
        go(at + 1);
        break;
      case "ArrowUp":
        go(at < 0 ? -1 : at - 1);
        break;
      case "Home":
        go(0);
        break;
      case "End":
        go(-1);
        break;
      case "ArrowRight":
        e.preventDefault();
        e.stopPropagation();
        if (current?.submenu && !current.disabled) openSub(current.id, "first");
        else onSide?.(1);
        break;
      case "ArrowLeft":
        e.preventDefault();
        e.stopPropagation();
        onSide?.(-1);
        break;
      case "Escape":
        e.preventDefault();
        e.stopPropagation();
        onClose("escape");
        break;
      case "Tab":
        onClose("tab");
        break;
    }
  };

  return (
    <div
      ref={menuRef}
      role="menu"
      tabIndex={-1}
      id={id}
      aria-label={label}
      aria-labelledby={labelledBy}
      className={className}
      data-state={state}
      style={style}
      onKeyDown={onKeyDown}
      onAnimationEnd={(e) => {
        if (e.target === e.currentTarget && state === "closed") onExited();
      }}
    >
      {items.map((item, i) => {
        const checkable = item.checked !== undefined;
        const nested = item.submenu;
        const expanded = sub?.id === item.id;
        const parts = [];
        if (item.separator && i > 0)
          parts.push(<div key={`${item.id}-sep`} role="separator" className="menu-separator" />);
        parts.push(
          <button
            key={item.id}
            type="button"
            role={checkable ? "menuitemradio" : "menuitem"}
            data-item={item.id}
            aria-checked={checkable ? item.checked : undefined}
            aria-disabled={item.disabled || undefined}
            title={item.title}
            aria-description={item.disabled ? item.title : undefined}
            aria-haspopup={nested ? "menu" : undefined}
            aria-expanded={nested ? expanded : undefined}
            tabIndex={-1}
            className="menu-item"
            style={{ "--i": i } as CSSProperties}
            onPointerEnter={() => {
              window.clearTimeout(hover.current);
              if (nested && expanded) return;
              hover.current = window.setTimeout(
                () => (nested && !item.disabled ? openSub(item.id, "menu") : setSub(null)),
                SUBMENU_DELAY_MS,
              );
            }}
            onClick={() => {
              if (item.disabled) return;
              if (nested) {
                if (expanded) setSub(null);
                else openSub(item.id, "first");
                return;
              }
              item.onSelect();
              onClose("select");
            }}
            onKeyDown={(e) => {
              if (nested && !item.disabled && (e.key === "Enter" || e.key === " ")) {
                e.preventDefault();
                openSub(item.id, "first");
              }
            }}
          >
            {checkable && (
              <span className="menu-check">{item.checked && <Icon name="check" />}</span>
            )}
            <span className="menu-label">{item.label}</span>
            {item.trailing}
            {item.shortcut && <span className="menu-shortcut">{item.shortcut}</span>}
            {nested && <Icon name="chevron" className="menu-sub-chevron" />}
          </button>,
        );
        if (nested && expanded) {
          parts.push(
            <MenuList
              key={`${item.id}-menu`}
              menuRef={subMenu}
              label={typeof item.label === "string" ? item.label : undefined}
              items={nested}
              state="open"
              focus={sub.focus}
              className="menu sub"
              style={{ top: sub.top }}
              onClose={(reason) => (reason === "escape" ? closeSub() : onClose(reason))}
              onExited={() => undefined}
              onSide={(direction) => (direction < 0 ? closeSub() : onSide?.(1))}
            />,
          );
        }
        return parts;
      })}
    </div>
  );
}

interface MenuProps {
  /** Trigger content; a chevron follows it and turns over while the menu is open. */
  label: ReactNode;
  items: MenuItem[];
  align?: "start" | "end";
  /** Opens upwards, for menus at the bottom of a window. */
  side?: "bottom" | "top";
  /** Classes of the trigger button. */
  triggerClass?: string;
  /** Accessible name and tooltip when the label is only an icon. */
  ariaLabel?: string;
  title?: string;
  chevron?: boolean;
  disabled?: boolean;
  /** The list appears (true) and has gone (false): a small window makes room for it. */
  onShown?: (shown: boolean) => void;
}

/** A button that opens a list of actions or choices below it (or above, with `side`). */
export function Menu({
  label,
  items,
  align = "start",
  side = "bottom",
  triggerClass = "btn",
  ariaLabel,
  title,
  chevron = true,
  disabled,
  onShown,
}: MenuProps) {
  const [open, setOpen] = useState(false);
  const [focus, setFocus] = useState<"first" | "menu">("menu");
  const presence = usePresence(open);
  const anchor = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const id = useId();
  const shownRef = useRef(onShown);
  useEffect(() => {
    shownRef.current = onShown;
  });
  useEffect(() => {
    shownRef.current?.(presence.mounted);
  }, [presence.mounted]);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (!anchor.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", onDown, true);
    return () => document.removeEventListener("pointerdown", onDown, true);
  }, [open]);

  const show = (how: "first" | "menu") => {
    setFocus(how);
    setOpen(true);
  };
  const close = (reason: CloseReason) => {
    setOpen(false);
    if (reason !== "outside" && reason !== "tab") trigger.current?.focus({ preventScroll: true });
  };

  return (
    <div className="menu-anchor" ref={anchor}>
      <button
        ref={trigger}
        type="button"
        className={triggerClass}
        id={`${id}-trigger`}
        aria-label={ariaLabel}
        title={title}
        disabled={disabled}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={open ? `${id}-menu` : undefined}
        onClick={() => (open ? setOpen(false) : show("menu"))}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown" || e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            show("first");
          }
        }}
      >
        {label}
        {chevron && <Icon name="chevron" className="chevron" />}
      </button>
      {presence.mounted && (
        <MenuList
          menuRef={menu}
          id={`${id}-menu`}
          labelledBy={`${id}-trigger`}
          items={items}
          state={presence.state}
          focus={focus}
          className={`menu${align === "end" ? " end" : ""}${side === "top" ? " up" : ""}`}
          onClose={close}
          onExited={presence.exited}
        />
      )}
    </div>
  );
}

interface ContextMenuProps {
  open: boolean;
  /** Pointer position in viewport coordinates (clientX / clientY). */
  x: number;
  y: number;
  /** Accessible name of the menu. */
  label: string;
  items: MenuItem[];
  onClose: (reason: CloseReason) => void;
}

/** A menu at the pointer; it flips to stay inside the window and grows from that corner. */
export function ContextMenu({ open, x, y, label, items, onClose }: ContextMenuProps) {
  const presence = usePresence(open);
  const menu = useRef<HTMLDivElement>(null);
  const closeRef = useRef(onClose);
  useEffect(() => {
    closeRef.current = onClose;
  });

  useLayoutEffect(() => {
    const el = menu.current;
    if (!open || !el) return;
    const flipX = x + el.offsetWidth > window.innerWidth - 8;
    const flipY = y + el.offsetHeight > window.innerHeight - 8;
    el.style.left = `${Math.max(8, flipX ? x - el.offsetWidth : x)}px`;
    el.style.top = `${Math.max(8, flipY ? y - el.offsetHeight : y)}px`;
    el.style.setProperty(
      "--menu-origin",
      `${flipY ? "bottom" : "top"} ${flipX ? "right" : "left"}`,
    );
  }, [open, x, y]);

  useEffect(() => {
    const el = menu.current;
    if (!open || !el) return;
    const previous = document.activeElement as HTMLElement | null;
    const onDown = (e: PointerEvent) => {
      if (!el.contains(e.target as Node)) closeRef.current("outside");
    };
    const onBlur = () => closeRef.current("blur");
    document.addEventListener("pointerdown", onDown, true);
    window.addEventListener("blur", onBlur);
    window.addEventListener("resize", onBlur);
    return () => {
      document.removeEventListener("pointerdown", onDown, true);
      window.removeEventListener("blur", onBlur);
      window.removeEventListener("resize", onBlur);
      if (el.contains(document.activeElement)) previous?.focus({ preventScroll: true });
    };
  }, [open]);

  if (!presence.mounted) return null;
  // Rendered on <body> so an animating or scrolling ancestor cannot move or clip it.
  return createPortal(
    <MenuList
      menuRef={menu}
      label={label}
      items={items}
      state={presence.state}
      focus="first"
      className="menu context"
      style={{ left: x, top: y }}
      onClose={(reason) => closeRef.current(reason)}
      onExited={presence.exited}
    />,
    document.body,
  );
}
