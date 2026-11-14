// The editor's workspace (Cristal, EDT-001, EDT-005, EDT-011, TRY-002): the canvas fills all
// of it and the toolbar, the side panel, the status bar and the tray float over it as glass,
// so the capture passes beneath them. The canvas gets how much each bar covers (its insets)
// and fits, centers and scrolls the image in the part they leave visible.
import { useCallback, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { SidePanel } from "./SidePanel";
import { StatusBar } from "./StatusBar";
import { Strip } from "./Strip";
import { Toolbar } from "./Toolbar";
import { ToolbarCustomizer } from "./ToolbarCustomizer";
import { VideoBar } from "./VideoBar";
import { insetsFrom, type Insets } from "./insets";

interface Sizes {
  top: number;
  side: number;
  bottom: number;
}

export function Workspace({ children }: { children: (insets: Insets) => ReactNode }) {
  const root = useRef<HTMLDivElement>(null);
  const top = useRef<HTMLDivElement>(null);
  const side = useRef<HTMLDivElement>(null);
  const bottom = useRef<HTMLDivElement>(null);
  const [sizes, setSizes] = useState<Sizes | null>(null);

  const measure = useCallback(() => {
    const next = {
      top: top.current?.offsetHeight ?? 0,
      side: side.current?.offsetWidth ?? 0,
      bottom: bottom.current?.offsetHeight ?? 0,
    };
    setSizes((s) =>
      s && s.top === next.top && s.side === next.side && s.bottom === next.bottom ? s : next,
    );
  }, []);

  // After every render (a bar may have appeared or gone) and whenever a bar changes size.
  useLayoutEffect(measure);
  useLayoutEffect(() => {
    const observer = new ResizeObserver(measure);
    for (const el of [top.current, side.current, bottom.current]) if (el) observer.observe(el);
    return () => observer.disconnect();
  }, [measure]);

  const insets = sizes ? insetsFrom(sizes.top, sizes.side, sizes.bottom) : null;

  // The editor's toasts and hints sit above the floating bars, so they read the insets too.
  useLayoutEffect(() => {
    const host = root.current?.parentElement;
    if (!host || !sizes) return;
    const { top, right, bottom } = insetsFrom(sizes.top, sizes.side, sizes.bottom);
    host.style.setProperty("--ws-top", `${top}px`);
    host.style.setProperty("--ws-side", `${right}px`);
    host.style.setProperty("--ws-bottom", `${bottom}px`);
    return () => {
      for (const name of ["--ws-top", "--ws-side", "--ws-bottom"]) host.style.removeProperty(name);
    };
  }, [sizes]);

  return (
    <div className="workspace" ref={root}>
      <div className="editor-main">{insets && children(insets)}</div>
      <div className="ws-top" ref={top}>
        <Toolbar />
        <ToolbarCustomizer />
      </div>
      <div className="ws-side" ref={side}>
        <SidePanel />
      </div>
      <div className="ws-bottom" ref={bottom}>
        {/* A video's playback bar (VID-056), over the status bar. */}
        <VideoBar />
        <StatusBar />
        <Strip />
      </div>
    </div>
  );
}
