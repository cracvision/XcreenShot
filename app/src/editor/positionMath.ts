// Position and size typed in numbers (EDT-086). A field takes a number, or an
// expression on its current value: `+10`, `-5`, `*2`, `/2`.

/** The value typed in a field: a number replaces, `+n` `-n` `*n` `/n` change the current
 *  value. Anything else (text, division by zero) is null, and the field goes back. */
export function evaluate(input: string, current: number): number | null {
  const s = input.trim().replace(",", ".");
  if (s === "") return null;
  const m = /^([+\-*/])\s*(\d+(?:\.\d+)?)$/.exec(s);
  if (m) {
    const n = Number(m[2]);
    switch (m[1]) {
      case "+":
        return current + n;
      case "-":
        return current - n;
      case "*":
        return current * n;
      case "/":
        return n === 0 ? null : current / n;
    }
  }
  // "=-5" sets a negative value (a leading minus alone takes 5 away).
  const v = Number(s.startsWith("=") ? s.slice(1) : s);
  return Number.isFinite(v) ? v : null;
}

/** New width and height when one changes with the proportion locked. */
export function keepRatio(
  w: number,
  h: number,
  changed: "w" | "h",
  value: number,
): { w: number; h: number } {
  if (changed === "w") return { w: value, h: w === 0 ? h : (h * value) / w };
  return { w: h === 0 ? w : (w * value) / h, h: value };
}
