# Emoji stamps (EDT-033)

`emoji.bin` and `emoji.json` are built by `scripts/make-emoji.py` from:

- **Noto Emoji** (https://github.com/googlefonts/noto-emoji): the images in `2D/png/128`, Apache License 2.0. Flags come from its `third_party/region-flags`, in the public domain.
- **Unicode CLDR** (https://github.com/unicode-org/cldr): the names and keywords in English and Spanish (`common/annotations` and `common/annotationsDerived`), Unicode License v3.

The images are resized to at most 128 px and stored as WebP. Skin-tone variants are left out.
