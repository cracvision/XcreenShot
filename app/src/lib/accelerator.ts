// Shortcuts are stored by physical key (KeyboardEvent.code) in the canonical form
// `Ctrl+Alt+Shift+Super+KeyK` (APP-017). This module converts key events to that
// form and renders it with the active keyboard layout when the platform exposes it.

const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "ShiftLeft",
  "ShiftRight",
  "MetaLeft",
  "MetaRight",
  "OSLeft",
  "OSRight",
  "CapsLock",
  "Fn",
]);

export interface KeyLike {
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

export function isModifierCode(code: string): boolean {
  return MODIFIER_CODES.has(code);
}

/** Canonical accelerator for a key event, or null while only modifiers are held. */
export function fromEvent(e: KeyLike): string | null {
  if (!e.code || isModifierCode(e.code)) return null;
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");
  parts.push(e.code);
  return parts.join("+");
}

/**
 * What a recorder takes when a key is released: the combination seen while pressing,
 * or, for PrtScn, the release itself (Windows only reports PrtScn on key-up).
 */
export function releasedCombination(candidate: string | null, e: KeyLike): string | null {
  if (candidate) return candidate;
  return e.code === "PrintScreen" ? fromEvent(e) : null;
}

const KEY_LABELS: Record<string, string> = {
  PrintScreen: "PrtScn",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Semicolon: ";",
  Quote: "'",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Minus: "-",
  Equal: "=",
  Backquote: "`",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
};

/** Layout map from `navigator.keyboard.getLayoutMap()` (Chromium/WebView2 only). */
export type LayoutMap = Pick<Map<string, string>, "get">;

export function keyLabel(code: string, layout?: LayoutMap): string {
  const fromLayout = layout?.get(code);
  if (fromLayout && fromLayout.trim() !== "") return fromLayout.toUpperCase();
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  return KEY_LABELS[code] ?? code;
}

export function display(
  canonical: string,
  platform: "windows" | "macos",
  layout?: LayoutMap,
): string {
  const parts = canonical.split("+");
  const code = parts.pop() ?? "";
  const mods = new Set(parts);
  const key = keyLabel(code, layout);
  if (platform === "macos") {
    return (
      (mods.has("Ctrl") ? "⌃" : "") +
      (mods.has("Alt") ? "⌥" : "") +
      (mods.has("Shift") ? "⇧" : "") +
      (mods.has("Super") ? "⌘" : "") +
      key
    );
  }
  const out: string[] = [];
  if (mods.has("Ctrl")) out.push("Ctrl");
  if (mods.has("Alt")) out.push("Alt");
  if (mods.has("Shift")) out.push("Shift");
  if (mods.has("Super")) out.push("Win");
  out.push(key);
  return out.join("+");
}

interface NavigatorKeyboard {
  keyboard?: { getLayoutMap?: () => Promise<LayoutMap> };
}

export async function loadLayoutMap(): Promise<LayoutMap | undefined> {
  try {
    return await (navigator as NavigatorKeyboard).keyboard?.getLayoutMap?.();
  } catch {
    return undefined;
  }
}
