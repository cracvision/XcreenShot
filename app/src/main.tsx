import React from "react";
import ReactDOM from "react-dom/client";
import "./i18n";
import "./styles.css";
import { App } from "./App";
import { useApp } from "./store";

void useApp.getState().boot();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
