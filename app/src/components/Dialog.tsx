import { useEffect, useRef, type ReactNode } from "react";

interface Props {
  title: string;
  children: ReactNode;
  actions: ReactNode;
  onCancel: () => void;
}

/** Modal dialog: focus moves inside, Esc cancels, the title labels it. */
export function Dialog({ title, children, actions, onCancel }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const cancel = useRef(onCancel);
  useEffect(() => {
    cancel.current = onCancel;
  });
  // Once, when it opens: a new onCancel at each redraw must not pull the focus back to
  // the first button while the user types in a field of the dialog.
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    // A field marked data-autofocus (what the dialog asks for), else the first button.
    ref.current?.querySelector<HTMLElement>("[data-autofocus], button")?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        cancel.current();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      previous?.focus();
    };
  }, []);
  return (
    <div className="backdrop">
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        ref={ref}
      >
        <h2 id="dialog-title">{title}</h2>
        {children}
        <div className="actions">{actions}</div>
      </div>
    </div>
  );
}
