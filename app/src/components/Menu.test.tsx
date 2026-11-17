import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ContextMenu, Menu, type MenuItem } from "./Menu";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

const choices = (onSelect: (id: string) => void): MenuItem[] =>
  ["All", "Global", "Custom"].map((label, i) => ({
    id: label,
    label,
    checked: i === 0,
    onSelect: () => onSelect(label),
  }));

describe("Menu (APP-043)", () => {
  it("opens from the keyboard on the first item and wraps with the arrows", () => {
    render(<Menu label="Show: All" items={choices(() => {})} />);
    const trigger = screen.getByRole("button", { name: "Show: All" });
    fireEvent.keyDown(trigger, { key: "ArrowDown" });
    expect(trigger.getAttribute("aria-expanded")).toBe("true");
    const [first, second, third] = screen.getAllByRole("menuitemradio") as [
      HTMLElement,
      HTMLElement,
      HTMLElement,
    ];
    expect(document.activeElement).toBe(first);
    expect(first.getAttribute("aria-checked")).toBe("true");
    fireEvent.keyDown(first, { key: "ArrowDown" });
    expect(document.activeElement).toBe(second);
    fireEvent.keyDown(second, { key: "End" });
    expect(document.activeElement).toBe(third);
    fireEvent.keyDown(third, { key: "ArrowDown" });
    expect(document.activeElement).toBe(first);
  });

  it("closes with Escape, returns focus and stays mounted until the exit animation ends", () => {
    vi.useFakeTimers();
    render(<Menu label="Show: All" items={choices(() => {})} />);
    const trigger = screen.getByRole("button", { name: "Show: All" });
    fireEvent.keyDown(trigger, { key: "Enter" });
    fireEvent.keyDown(screen.getAllByRole("menuitemradio")[0] as HTMLElement, { key: "Escape" });
    expect(document.activeElement).toBe(trigger);
    expect(trigger.getAttribute("aria-expanded")).toBe("false");
    expect(screen.getByRole("menu").getAttribute("data-state")).toBe("closed");
    act(() => {
      vi.advanceTimersByTime(400);
    });
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("runs the chosen item and closes", () => {
    const onSelect = vi.fn();
    render(<Menu label="Show: All" items={choices(onSelect)} />);
    const trigger = screen.getByRole("button", { name: "Show: All" });
    fireEvent.click(trigger);
    fireEvent.click(screen.getByRole("menuitemradio", { name: "Custom" }));
    expect(onSelect).toHaveBeenCalledExactlyOnceWith("Custom");
    expect(trigger.getAttribute("aria-expanded")).toBe("false");
  });
});

describe("ContextMenu", () => {
  it("focuses its first item and reports Escape", () => {
    const onClose = vi.fn();
    render(
      <ContextMenu
        open
        x={10}
        y={10}
        label="Theme"
        onClose={onClose}
        items={[{ id: "copy", label: "Copy key", onSelect: () => {} }]}
      />,
    );
    const item = screen.getByRole("menuitem", { name: "Copy key" });
    expect(document.activeElement).toBe(item);
    fireEvent.keyDown(item, { key: "Escape" });
    expect(onClose).toHaveBeenCalledWith("escape");
  });
});
