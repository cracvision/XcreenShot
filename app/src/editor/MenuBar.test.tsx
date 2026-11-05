import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import "../i18n";
import { useEditorKeys } from "./keys";
import { MenuBar } from "./MenuBar";
import { useMenuState } from "./menus";

const command = (id: string, label: string, action: string, checked: string | null = null) => ({
  kind: "command",
  id,
  label,
  command: "editor.action",
  params: { action },
  checked,
});

const MENUS = [
  {
    kind: "submenu",
    label: "menu.file",
    items: [
      command("editor.open", "menu.open", "open"),
      { kind: "separator" },
      command("editor.save", "menu.save", "save"),
    ],
  },
  {
    kind: "submenu",
    label: "menu.view",
    items: [
      {
        kind: "submenu",
        label: "menu.toolbar",
        items: [
          command(
            "editor.toolbar_labels",
            "menu.toolbar_labels",
            "toolbar_labels",
            "editor.toolbar_labels",
          ),
        ],
      },
    ],
  },
];

const run = vi.hoisted(() => vi.fn(() => Promise.resolve({})));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve(MENUS)) }));
vi.mock("../lib/commands", () => ({ run }));

beforeEach(() => {
  run.mockClear();
  useMenuState.setState({
    enabled: { "editor.save": false },
    checked: { "editor.toolbar_labels": true },
  });
});
afterEach(cleanup);

/** The bar's top entries (File, View), once the menus have loaded. */
async function mount(): Promise<[HTMLElement, HTMLElement]> {
  render(<MenuBar />);
  await act(async () => {});
  const [file, view] = screen
    .getAllByRole("menuitem")
    .filter((b) => b.classList.contains("menubar-top"));
  if (!file || !view) throw new Error("menu bar not drawn");
  return [file, view];
}

const focused = () => document.activeElement as HTMLElement;

describe("drawn menu bar (EDT-002)", () => {
  it("F10 reaches the bar; the arrows move along it and open a menu", async () => {
    const [file, view] = await mount();
    fireEvent.keyDown(window, { key: "F10" });
    expect(focused()).toBe(file);
    fireEvent.keyDown(file, { key: "ArrowRight" });
    expect(focused()).toBe(view);
    fireEvent.keyDown(view, { key: "ArrowLeft" });
    fireEvent.keyDown(file, { key: "ArrowDown" });
    expect(file.getAttribute("aria-expanded")).toBe("true");
    expect(focused().textContent).toBe("Open…");
  });

  it("Alt alone reaches the bar; Alt plus a menu's letter opens it", async () => {
    const [file, view] = await mount();
    fireEvent.keyDown(window, { key: "Alt" });
    fireEvent.keyUp(window, { key: "Alt" });
    expect(focused()).toBe(file);
    expect(file.textContent).toBe("File");
    fireEvent.keyDown(window, { key: "Escape" });
    fireEvent.keyDown(window, { key: "v", altKey: true });
    expect(view.getAttribute("aria-expanded")).toBe("true");
    expect(view.getAttribute("aria-keyshortcuts")).toBe("Alt+V");
  });

  it("runs the entry's command; a disabled entry stays visible and does nothing", async () => {
    const [file] = await mount();
    fireEvent.click(file);
    const save = screen.getByText("Save").closest("button");
    expect(save?.getAttribute("aria-disabled")).toBe("true");
    if (save) fireEvent.click(save);
    expect(run).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("Open…"));
    expect(run).toHaveBeenCalledWith("editor.action", { action: "open" });
  });

  it("Right opens the next menu; a submenu opens with Right and closes with Left", async () => {
    const [file, view] = await mount();
    fireEvent.keyDown(file, { key: "ArrowDown" });
    fireEvent.keyDown(focused(), { key: "ArrowRight" });
    expect(view.getAttribute("aria-expanded")).toBe("true");
    const toolbar = focused();
    expect(toolbar.textContent).toContain("Toolbar");
    fireEvent.keyDown(toolbar, { key: "ArrowRight" });
    const labels = focused();
    expect(labels.textContent).toContain("Show labels");
    expect(labels.getAttribute("aria-checked")).toBe("true");
    fireEvent.keyDown(labels, { key: "ArrowLeft" });
    expect(focused()).toBe(toolbar);
    fireEvent.keyDown(toolbar, { key: "Escape" });
    expect(focused()).toBe(view);
  });

  it("the editor's own keys leave the bar's arrows alone", async () => {
    function WithEditorKeys() {
      useEditorKeys();
      return <MenuBar />;
    }
    render(<WithEditorKeys />);
    await act(async () => {});
    const [file, edit] = screen
      .getAllByRole("menuitem")
      .filter((b) => b.classList.contains("menubar-top"));
    fireEvent.keyDown(window, { key: "F10" });
    expect(focused()).toBe(file);
    fireEvent.keyDown(focused(), { key: "ArrowRight", code: "ArrowRight" });
    expect(focused()).toBe(edit);
  });
});
