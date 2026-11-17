// How the Library writes sizes and dates (LIB-004, LIB-012).

/** Bytes as B, KB, MB or GB, in the language's number format. */
export function fileSize(bytes: number, lang: string): string {
  const units = ["B", "KB", "MB", "GB"];
  let v = bytes;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) {
    v /= 1024;
    u += 1;
  }
  return `${v.toLocaleString(lang, { maximumFractionDigits: u === 0 ? 0 : 1 })} ${units[u]}`;
}

/** A date and time, or "—" when there is none (LIB-012). */
export function when(iso: string | null | undefined, lang: string): string {
  if (!iso) return "—";
  const d = new Date(iso);
  return Number.isNaN(d.getTime())
    ? "—"
    : d.toLocaleString(lang, { dateStyle: "medium", timeStyle: "short" });
}
