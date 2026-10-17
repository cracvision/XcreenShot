import React from "react";
import ReactDOM from "react-dom/client";
import "./i18n";
import "./styles.css";
import { App } from "./App";
import { isOverlay } from "./windowKind";
import { useApp } from "./store";

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
