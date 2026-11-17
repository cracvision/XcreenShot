"""Builds the Emoji stamps (EDT-033) from Noto Emoji and Unicode CLDR.

One-off tool, run by hand when the emoji set is updated; needs Python 3 and Pillow.

    python scripts/make-emoji.py <noto-emoji checkout> <cldr annotations folder>

- <noto-emoji checkout>: https://github.com/googlefonts/noto-emoji (Apache 2.0 for
  the images; flags from third_party/region-flags, public domain). Only 2D/png/128
  and third_party/region-flags/png are read.
- <cldr annotations folder>: annotations_{en,es}.xml and annotationsDerived_{en,es}.xml
  from https://github.com/unicode-org/cldr/tree/main/common (Unicode license).

Writes crates/xs-image/assets/emoji.bin (the pictures, WebP, one after another) and
crates/xs-image/assets/emoji.json (for each: id, character, names and keywords in
English and Spanish, and where its picture is in the .bin). Skin-tone variants are
left out: each emoji comes in its default look (provisional, spec/QUESTIONS.md).
"""

import io
import json
import os
import re
import sys
import xml.etree.ElementTree as ET

from PIL import Image

SIZE = 128
QUALITY = 82
TONES = {0x1F3FB, 0x1F3FC, 0x1F3FD, 0x1F3FE, 0x1F3FF}


def annotations(folder, lang):
    """character -> (name, keywords) from CLDR, derived ones too."""
    out = {}
    for kind in ("annotations", "annotationsDerived"):
        path = os.path.join(folder, f"{kind}_{lang}.xml")
        for a in ET.parse(path).getroot().iter("annotation"):
            cp = a.get("cp")
            name, words = out.get(cp, ("", ""))
            if a.get("type") == "tts":
                name = a.text or ""
            else:
                words = a.text or ""
            out[cp] = (name, words)
    return out


def webp(img):
    img = img.convert("RGBA")
    img.thumbnail((SIZE, SIZE), Image.LANCZOS)
    buf = io.BytesIO()
    img.save(buf, "WEBP", quality=QUALITY, method=6)
    return buf.getvalue(), img.size


def main():
    noto, cldr = sys.argv[1], sys.argv[2]
    en, es = annotations(cldr, "en"), annotations(cldr, "es")
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets = os.path.join(root, "crates", "xs-image", "assets")
    os.makedirs(assets, exist_ok=True)
    entries = []
    blob = bytearray()

    def add(chars, img):
        # CLDR names the fully qualified form; Noto files drop U+FE0F.
        key = chars if chars in en else chars + "️" if chars + "️" in en else None
        if key is None:
            key = next((k for k in en if k.replace("️", "") == chars), None)
        if key is None:
            return
        data, (w, h) = webp(img)
        entries.append(
            {
                "id": "-".join(f"{ord(c):x}" for c in key if ord(c) != 0xFE0F),
                "char": key,
                "en": en[key][0],
                "es": es.get(key, ("", ""))[0] or en[key][0],
                "keywordsEn": en[key][1],
                "keywordsEs": es.get(key, ("", ""))[1],
                "w": w,
                "h": h,
                "at": len(blob),
                "len": len(data),
            }
        )
        blob.extend(data)

    pngs = os.path.join(noto, "2D", "png", "128")
    for name in sorted(os.listdir(pngs)):
        m = re.fullmatch(r"emoji_u([0-9a-f_]+)\.png", name)
        if not m:
            continue
        cps = [int(x, 16) for x in m.group(1).split("_")]
        if any(c in TONES for c in cps):
            continue
        add("".join(chr(c) for c in cps), Image.open(os.path.join(pngs, name)))

    flags = os.path.join(noto, "third_party", "region-flags", "png")
    for name in sorted(os.listdir(flags)):
        m = re.fullmatch(r"([A-Z]{2})\.png", name)
        if not m:
            continue
        chars = "".join(chr(0x1F1E6 + ord(c) - ord("A")) for c in m.group(1))
        path = os.path.join(flags, name)
        # A few flags are links to another one (a checkout on Windows keeps the
        # link as a small text file with the target's name).
        if os.path.getsize(path) < 64:
            with open(path, encoding="ascii", errors="ignore") as f:
                path = os.path.join(flags, f.read().strip())
        add(chars, Image.open(path))

    with open(os.path.join(assets, "emoji.bin"), "wb") as f:
        f.write(blob)
    with open(os.path.join(assets, "emoji.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, separators=(",", ":"))
        f.write("\n")
    print(len(entries), "emoji,", len(blob) // 1024, "KB")


if __name__ == "__main__":
    main()
