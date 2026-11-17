// QR codes read on this computer (SMT-038), shared by the editor and the selection
// on screen.

export interface QrCode {
  text: string;
  /** Where it is: in the image, or on the virtual screen (physical px) in a capture. */
  rect: [number, number, number, number];
  kind: "url" | "wifi" | "text";
  host?: string;
  ssid?: string;
  security?: string;
  password?: string;
}

/** The address with its domain apart, to show before opening it. */
export function splitUrl(url: string): [string, string, string] {
  const m = /^([a-z][a-z0-9+.-]*:\/\/)([^/?#]*)(.*)$/i.exec(url);
  return m ? [m[1] ?? "", m[2] ?? "", m[3] ?? ""] : ["", url, ""];
}

/** The codes whose middle is inside `area` (same coordinates). */
export function codesIn(
  codes: QrCode[],
  area: { x: number; y: number; width: number; height: number },
): QrCode[] {
  return codes.filter(({ rect: [x, y, w, h] }) => {
    const cx = x + w / 2;
    const cy = y + h / 2;
    return cx >= area.x && cx <= area.x + area.width && cy >= area.y && cy <= area.y + area.height;
  });
}
