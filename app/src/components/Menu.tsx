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

export interface MenuItem {
  id: string;
  label: ReactNode;
  /** Set (true or false) for a choice that shows a check mark when chosen. */
  checked?: boolean;
  disabled?: boolean;
  /** Extra content after the label, such as a setting key. */
  trailing?: ReactNode;
  /** Shortcut shown at the right, already written for this platform. */
  shortcut?: string;
  /** Draws a separator above the item. */
  separator?: boolean;
  onSelect: () => void;
}

export type CloseReason = "select" | "escape" | "tab" | "outside" | "blur";

/** Upper bound for the exit animation, in case animationend never fires. */
const EXIT_MS = 320;

/** Keeps a closing menu mounted until its exit animation has played. */
function usePresence(open: boolean) {
  const [closing, setClosing] = useState(false);
  const [wasOpen, setWasOpen] = useState(open);
  if (open !== wasOpen) {
    setWasOpen(open);
    setClosing(!open);
  }
  useEffect(() => {
    if (!closing) return;
    const timer = window.setTimeout(() => setClosing(false), EXIT_MS);
    return () => window.clearTimeout(timer);
  }, [closing]);
  return {
    mounted: open || closing,
    state: open ? ("open" as const) : ("closed" as const),
    exited: () => setClosing(false),
  };
}

function enabledItems(menu: HTMLElement): HTMLElement[] {
  return Array.from(
    menu.querySelectorAll<HTMLElement>('[role^="menuitem"]:not([aria-disabled="true"])'),
  );
}

interface MenuListProps {
  items: MenuItem[];
  state: "open" | "closed";
  /** Where focus goes when the menu opens. */
  focus: "first" | "menu";
  className: string;
  menuRef: RefObject<HTMLDivElement>;
  id?: string;
  label?: string;
  labelledBy?: string;
  style?: CSSProperties;
  onClose: (reason: CloseReason) => void;
  onExited: () => void;
}

function MenuList({
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
}: MenuListProps) {
  useEffect(() => {
    const menu = menuRef.current;
    if (state !== "open" || !menu) return;
    if (focus === "first") enabledItems(menu)[0]?.focus({ preventScroll: true });
    else menu.focus({ preventScroll: true });
  }, [state, focus, menuRef]);

  const onKeyDown = (e: ReactKeyboardEvent<HTMLDivElement>) => {
    const list = enabledItems(e.currentTarget);
    const at = list.indexOf(document.activeElement as HTMLElement);
    const go = (n: number) => {
      e.preventDefault();
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
        const button = (
          <button
            key={item.id}
            type="button"
            role={checkable ? "menuitemradio" : "menuitem"}
            aria-checked={checkable ? item.checked : undefined}
            aria-disabled={item.disabled || undefined}
            tabIndex={-1}
            className="menu-item"
            style={{ "--i": i } as CSSProperties}
            onClick={() => {
              if (item.disabled) return;
              item.onSelect();
              onClose("select");
            }}
          >
            {checkable && (
              <span className="menu-check">{item.checked && <Icon name="check" />}</span>
            )}
            <span className="menu-label">{item.label}</span>
            {item.trailing}
            {item.shortcut && <span className="menu-shortcut">{item.shortcut}</span>}
          </button>
        );
        if (!item.separator || i === 0) return button;
        return [<div key={`${item.id}-sep`} role="separator" className="menu-separator" />, button];
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
}: MenuProps) {
  const [open, setOpen] = useState(false);
  const [focus, setFocus] = useState<"first" | "menu">("menu");
  const presence = usePresence(open);
  const anchor = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const id = useId();

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
