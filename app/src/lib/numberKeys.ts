// Number fields (APP-043): the arrows add or take 1, as the field does by itself, and
// with Shift they add or take 10, within the field's min and max.
import type { KeyboardEvent } from "react";

/** Handles Shift+↑ and Shift+↓ in a number field; `apply` gets the new value. */
export function tenStep(e: KeyboardEvent<HTMLInputElement>, apply: (value: number) => void): void {
  if (!e.shiftKey || (e.key !== "ArrowUp" && e.key !== "ArrowDown")) return;
  e.preventDefault();
  const input = e.currentTarget;
  let next = (Number(input.value) || 0) + (e.key === "ArrowUp" ? 10 : -10);
  if (input.min !== "") next = Math.max(Number(input.min), next);
  if (input.max !== "") next = Math.min(Number(input.max), next);
  apply(next);
}
