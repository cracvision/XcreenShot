// Export formats and their options (EDT-111, EDT-112): what the Export dialog offers
// and remembers per format (in this window, like the last format).
import { create } from "zustand";

export const FORMATS = ["png", "jpg", "gif", "bmp", "tiff", "pdf", "ico", "psd"] as const;
export type ExportFormat = (typeof FORMATS)[number];
/** Formats of the spec that need a codec this build does not have. */
export const UNAVAILABLE = ["webp", "heic"] as const;

export const EXTENSIONS: Record<ExportFormat, string[]> = {
  png: ["png"],
  jpg: ["jpg", "jpeg"],
  gif: ["gif"],
  bmp: ["bmp"],
  tiff: ["tif", "tiff"],
  pdf: ["pdf"],
  ico: ["ico"],
  psd: ["psd"],
};

export const ICO_SIZES = [16, 24, 32, 48, 64, 128, 256];

export interface ExportOptions {
  quality: number;
  pngCompression: "fast" | "max";
  pngColors256: boolean;
  gifTransparent: boolean;
  tiffCompression: "none" | "lzw" | "zip";
  pdfPage: "auto" | "a4" | "letter" | "legal" | "a3";
  pdfOrientation: "auto" | "portrait" | "landscape";
  pdfMargin: number;
  pdfFitWidth: boolean;
  pdfSinglePage: boolean;
  pdfLossless: boolean;
  icoSizes: number[];
  scale: number;
  colorProfile: boolean;
}

export const DEFAULT_OPTIONS: ExportOptions = {
  quality: 90,
  pngCompression: "fast",
  pngColors256: false,
  gifTransparent: true,
  tiffCompression: "lzw",
  pdfPage: "auto",
  pdfOrientation: "auto",
  pdfMargin: 10,
  pdfFitWidth: true,
  pdfSinglePage: false,
  pdfLossless: true,
  icoSizes: [16, 32, 48, 256],
  scale: 100,
  colorProfile: true,
};

const FORMAT_KEY = "xs.export.format";
const OPTIONS_KEY = "xs.export.options";

export function lastExportFormat(): ExportFormat {
  try {
    const v = localStorage.getItem(FORMAT_KEY);
    if (v && (FORMATS as readonly string[]).includes(v)) return v as ExportFormat;
  } catch {
    // Storage unavailable.
  }
  return "png";
}

/** The options remembered for `format`, over the defaults. */
export function rememberedOptions(format: ExportFormat): ExportOptions {
  try {
    const all = JSON.parse(localStorage.getItem(OPTIONS_KEY) ?? "{}") as Record<
      string,
      Partial<ExportOptions>
    >;
    return { ...DEFAULT_OPTIONS, ...(all[format] ?? {}) };
  } catch {
    return { ...DEFAULT_OPTIONS };
  }
}

export function rememberExport(format: ExportFormat, options: ExportOptions): void {
  try {
    const all = JSON.parse(localStorage.getItem(OPTIONS_KEY) ?? "{}") as Record<string, unknown>;
    all[format] = options;
    localStorage.setItem(OPTIONS_KEY, JSON.stringify(all));
    localStorage.setItem(FORMAT_KEY, format);
  } catch {
    // Not remembered: no harm.
  }
}

/** Human size: bytes, KB or MB. */
export function fileSize(bytes: number, locale = "en"): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  const mb = Math.round((bytes / (1024 * 1024)) * 10) / 10;
  return `${mb.toLocaleString(locale, { maximumFractionDigits: 1 })} MB`;
}

/** The documents the Export dialog is open for, or null when closed. */
export const useExportDialog = create<{ ids: string[] | null }>(() => ({ ids: null }));
