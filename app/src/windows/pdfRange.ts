// Page ranges of Import PDF (CAP-041): "1-3, 7" and back.

/** The pages of `text` (1-based, each once, in order); null if it does not fit `count`. */
export function parseRange(text: string, count: number): number[] | null {
  if (!text.trim()) return Array.from({ length: count }, (_, i) => i + 1);
  const pages: number[] = [];
  for (const raw of text.split(/[,;]/)) {
    const part = raw.trim();
    if (!part) continue;
    const m = /^(\d+)\s*(?:-\s*(\d*))?$/.exec(part);
    if (!m) return null;
    const a = Number(m[1]);
    const b = m[2] === undefined ? a : m[2] === "" ? count : Number(m[2]);
    if (a < 1 || b < 1 || a > count || b > count || a > b) return null;
    for (let p = a; p <= b; p++) if (!pages.includes(p)) pages.push(p);
  }
  return pages.length ? pages : null;
}

/** The pages as a short range: runs become "a-b". */
export function formatRange(pages: number[]): string {
  const sorted = [...new Set(pages)].sort((a, b) => a - b);
  const parts: string[] = [];
  let i = 0;
  while (i < sorted.length) {
    let j = i;
    while (j + 1 < sorted.length && sorted[j + 1] === (sorted[j] ?? 0) + 1) j++;
    parts.push(i === j ? `${sorted[i]}` : `${sorted[i]}-${sorted[j]}`);
    i = j + 1;
  }
  return parts.join(", ");
}
