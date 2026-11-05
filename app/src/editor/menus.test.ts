import { beforeEach, describe, expect, it } from "vitest";
import { canvasMenu, closeContextMenu, toolbarMenu, useContextMenu } from "./menus";

beforeEach(() => closeContextMenu());

describe("editor right-click menus (EDT-002)", () => {
  it("on an object: cut, copy, then delete after a separator, at the pointer", () => {
    canvasMenu(true, { x: 120, y: 80 });
    const s = useContextMenu.getState();
    expect(s.open).toBe(true);
    expect([s.x, s.y]).toEqual([120, 80]);
    expect(s.items.map((i) => i.id)).toEqual(["editor.cut", "editor.copy", "editor.delete"]);
    expect(s.items.map((i) => i.separator)).toEqual([false, false, true]);
  });

  it("on the image: paste, paste in place and copy all", () => {
    canvasMenu(false, { x: 0, y: 0 });
    expect(useContextMenu.getState().items.map((i) => i.id)).toEqual([
      "editor.paste",
      "editor.paste_in_place",
      "editor.copy_all",
    ]);
  });

  it("on the toolbar: Icons only, as a choice with a check", () => {
    toolbarMenu({ x: 10, y: 10 });
    const [item] = useContextMenu.getState().items;
    expect(item?.id).toBe("editor.icons_only");
    expect(typeof item?.checked).toBe("boolean");
    closeContextMenu();
    expect(useContextMenu.getState().open).toBe(false);
  });
});
