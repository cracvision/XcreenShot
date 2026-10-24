import { useEffect, useRef, useState, type ReactNode } from "react";

export interface MenuItem {
  label: string;
  onSelect: () => void;
  disabled?: boolean;
  /** Draws a separator above the item. */
  separator?: boolean;
}

interface Props {
  label: ReactNode;
  /** Accessible name when `label` is only an icon. */
  ariaLabel?: string;
  title?: string;
  className?: string;
  items: MenuItem[];
  /** Opens upwards (menus at the bottom of the window). */
  up?: boolean;
  disabled?: boolean;
}

/** Button with a drop-down menu: arrows move, Esc closes, a click outside closes. */
export function MenuButton({ label, ariaLabel, title, className, items, up, disabled }: Props) {
  const [open, setOpen] = useState(false);
  const button = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    menu.current?.querySelector<HTMLElement>("[role=menuitem]:not(:disabled)")?.focus();
    const close = (e: MouseEvent) => {
      const target = e.target as Node;
      if (!menu.current?.contains(target) && !button.current?.contains(target)) setOpen(false);
    };
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [open]);

  const onKey = (e: React.KeyboardEvent) => {
    const all = [...(menu.current?.querySelectorAll<HTMLButtonElement>("[role=menuitem]") ?? [])];
    const enabled = all.filter((b) => !b.disabled);
    const i = enabled.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === "Escape") {
      e.stopPropagation();
      setOpen(false);
      button.current?.focus();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const d = e.key === "ArrowDown" ? 1 : -1;
      enabled[(i + d + enabled.length) % enabled.length]?.focus();
    }
  };

  return (
    <div className="menu-anchor">
      <button
        ref={button}
        type="button"
        className={className ?? "btn"}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={ariaLabel}
        title={title}
        disabled={disabled}
        onClick={() => setOpen((o) => !o)}
      >
        {label}
      </button>
      {open && (
        <div className={`menu${up ? " up" : ""}`} role="menu" ref={menu} onKeyDown={onKey}>
          {items.map((item, i) => (
            <div key={i} className="menu-row">
              {item.separator && <hr className="menu-separator" />}
              <button
                type="button"
                role="menuitem"
                tabIndex={-1}
                disabled={item.disabled}
                onClick={() => {
                  setOpen(false);
                  button.current?.focus();
                  item.onSelect();
                }}
              >
                {item.label}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
