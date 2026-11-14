import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Dialog } from "./Dialog";

afterEach(cleanup);

/** A dialog with a field, whose onCancel is a new function at each redraw. */
function Named({ onCancel }: { onCancel: () => void }) {
  const [name, setName] = useState("a");
  return (
    <Dialog title="Rename" onCancel={() => onCancel()} actions={<button type="button">OK</button>}>
      <input aria-label="Name" value={name} onChange={(e) => setName(e.target.value)} />
    </Dialog>
  );
}

describe("Dialog", () => {
  it("focuses its first button when it opens, and only then", () => {
    render(<Named onCancel={() => undefined} />);
    expect(document.activeElement).toBe(screen.getByText("OK"));
    const field = screen.getByLabelText("Name");
    field.focus();
    // Typing redraws the dialog: the focus stays in the field (TRY-006).
    fireEvent.change(field, { target: { value: "ab" } });
    fireEvent.change(field, { target: { value: "abc" } });
    expect(document.activeElement).toBe(field);
  });

  it("Escape cancels with the latest handler", () => {
    const onCancel = vi.fn();
    render(<Named onCancel={onCancel} />);
    fireEvent.change(screen.getByLabelText("Name"), { target: { value: "x" } });
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
