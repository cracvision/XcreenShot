// Images of the objects Rust draws for the canvas (text, blur), ADR 0004, and the
// pictures of floating images (EDT-078).
import { useEffect, useState } from "react";
import { fingerprint } from "./objects";
import { imageUrl, objectUrl, type DocObject } from "./state";

const images = new Map<string, HTMLImageElement>();

/**
 * Images of the objects Rust draws: object id → image. While a new version loads, the
 * previous image stays, so nothing flickers.
 */
export function useObjectImages(
  documentId: string,
  objects: DocObject[],
  version: number,
): Map<string, HTMLImageElement> {
  const [last, setLast] = useState(new Map<string, HTMLImageElement>());
  const wanted = objects.flatMap((o) => {
    // A picture never changes: its name is its content.
    if (o.type === "image") return [{ id: o.id, url: imageUrl(documentId, o.image) }];
    if (
      o.type !== "text" &&
      o.type !== "step" &&
      o.type !== "callout" &&
      o.type !== "pen" &&
      o.type !== "magnify" &&
      o.type !== "spotlight" &&
      o.type !== "stamp" &&
      o.type !== "simplify" &&
      !(o.type === "blur" && o.style.blurType !== "solid")
    )
      return [];
    return [
      {
        id: o.id,
        // A blur shows what is below it: anything changing below changes it.
        url: objectUrl(
          documentId,
          o.id,
          o.type === "blur" || o.type === "magnify" || o.type === "spotlight"
            ? `${version}-${fingerprint(o)}`
            : fingerprint(o),
        ),
      },
    ];
  });
  const key = wanted.map((w) => w.url).join("|");
  useEffect(() => {
    let alive = true;
    for (const { id, url } of wanted) {
      if (images.has(url)) continue;
      const el = new window.Image();
      el.crossOrigin = "anonymous";
      el.onload = () => {
        images.set(url, el);
        if (alive) setLast((m) => new Map(m).set(id, el));
      };
      el.src = url;
    }
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);
  const out = new Map<string, HTMLImageElement>();
  for (const { id, url } of wanted) {
    const image = images.get(url) ?? last.get(id);
    if (image) out.set(id, image);
  }
  return out;
}
