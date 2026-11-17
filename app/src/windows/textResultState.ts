// The result an Extract text window shows (SMT-006), and its tips when no text came.
import type { TFunction } from "i18next";
import type { QrCode } from "../lib/qr";

export interface TextResultData {
  id: string;
  found: boolean;
  words: number;
  language: string;
  text: string;
  textJoined: string;
  html: string;
  htmlJoined: string;
  /** The zone's QR codes, listed at the end (SMT-038). */
  codes?: QrCode[];
}

/** What to try when no text was found (SMT-006), in the app's own words. */
export function textTips(t: TFunction): string[] {
  return ["contrast", "straight", "frame", "letters", "language"].map((k) =>
    t(`ui.extract.tip_${k}`),
  );
}
