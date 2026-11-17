// Accessibility audit of the page it runs in (APP-043, APP-044): controls without an
// accessible name, form fields without a label, images without alt text, positive
// tabindex, and buttons that only show an icon without a tooltip. Run it in a window
// of an isolated instance:
//
//   XS_CDP_WINDOW=preferences node scripts/dev/cdp.mjs eval "$(cat scripts/dev/a11y-audit.js)"
//
// It returns a JSON list of problems (empty when there are none).
(() => {
  const visible = (el) => {
    const r = el.getBoundingClientRect();
    const st = getComputedStyle(el);
    return r.width > 0 && r.height > 0 && st.visibility !== "hidden" && st.display !== "none";
  };
  const text = (el) => (el.textContent || "").replace(/\s+/g, " ").trim();
  const nameOf = (el) => {
    const label = el.getAttribute("aria-label");
    if (label && label.trim()) return label.trim();
    const by = el.getAttribute("aria-labelledby");
    if (by) {
      const t = by
        .split(/\s+/)
        .map((id) => document.getElementById(id))
        .filter(Boolean)
        .map(text)
        .join(" ");
      if (t) return t;
    }
    if (el.id) {
      const l = document.querySelector(`label[for="${CSS.escape(el.id)}"]`);
      if (l && text(l)) return text(l);
    }
    const wrap = el.closest("label");
    if (wrap && text(wrap)) return text(wrap);
    if (el.tagName === "INPUT" && ["button", "submit"].includes(el.type) && el.value) return el.value;
    if (el.tagName === "IMG") return el.getAttribute("alt") ?? "";
    const own = text(el);
    if (own) return own;
    const img = el.querySelector("img[alt]");
    if (img && img.getAttribute("alt")) return img.getAttribute("alt");
    return (el.getAttribute("title") || "").trim();
  };
  const where = (el) => {
    const parts = [];
    for (let e = el; e && e !== document.body && parts.length < 4; e = e.parentElement) {
      let p = e.tagName.toLowerCase();
      if (e.id) p += `#${e.id}`;
      else if (e.classList.length) p += `.${[...e.classList].slice(0, 2).join(".")}`;
      parts.unshift(p);
    }
    return parts.join(" > ");
  };
  const problems = [];
  const controls = document.querySelectorAll(
    'button, a[href], input:not([type="hidden"]), select, textarea, [role="button"], [role="option"], [role="menuitem"], [role="menuitemcheckbox"], [role="switch"], [role="tab"], [role="slider"], [tabindex]:not([tabindex="-1"])',
  );
  for (const el of controls) {
    if (!visible(el)) continue;
    if (el.closest('[aria-hidden="true"]')) continue;
    const name = nameOf(el);
    if (!name) problems.push({ problem: "no accessible name", at: where(el), html: el.outerHTML.slice(0, 120) });
    const tab = Number(el.getAttribute("tabindex"));
    if (tab > 0) problems.push({ problem: "positive tabindex", at: where(el) });
    // An icon-only button needs a tooltip as well as a name (APP-014).
    if (el.tagName === "BUTTON" && !text(el) && !el.getAttribute("title") && !el.closest('[role="menu"]'))
      problems.push({ problem: "icon button without tooltip", at: where(el), name });
  }
  for (const img of document.querySelectorAll("img")) {
    if (visible(img) && !img.hasAttribute("alt"))
      problems.push({ problem: "image without alt", at: where(img) });
  }
  return JSON.stringify(problems, null, 1);
})()
