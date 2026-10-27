import React from "react";
import ReactDOM from "react-dom/client";
import "./i18n";
import "./styles.css";
import { App } from "./App";
import { isOverlay } from "./windowKind";
import { useApp } from "./store";

// The web view's own menu (Reload, Copy image of a canvas layer…) has no place in the
// app; text fields keep theirs for Cut, Copy and Paste.
window.addEventListener("contextmenu", (e) => {
  const t = e.target;
  const editable =
    t instanceof HTMLInputElement ||
    t instanceof HTMLTextAreaElement ||
    (t instanceof HTMLElement && t.isContentEditable);
  if (!editable) e.preventDefault();
});

// Selection windows get everything they need with each session (CAP-013).
if (isOverlay) {
  document.documentElement.classList.add("overlay-root");
} else {
  void useApp.getState().boot();
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
