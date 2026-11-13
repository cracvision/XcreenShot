// Thin scroll bars over the canvas (Cristal, EDT-007): the canvas fills the whole workspace,
// so the native ones would sit under the floating bars. They show in the part of the canvas
// left visible, only while the image does not fit, and widen under the pointer.
import { useRef, useState, type CSSProperties, type RefObject } from "react";
import { scrollForDrag, thumbOf, type Insets } from "./insets";

interface Size {
  width: number;
  height: number;
}

interface Props {
  /** The element whose scroll position is the canvas's. */
  scroller: RefObject<HTMLDivElement>;
  content: Size;
  visible: Size;
  scroll: { left: number; top: number };
  insets: Insets;
}

/** Room between a bar and the visible part's edges, and the bars' thickness. */
const PAD = 4;
const THICK = 12;

export function CanvasBars({ scroller, content, visible, scroll, insets }: Props) {
  const hTrack = visible.width - PAD * 2 - THICK;
  const vTrack = visible.height - PAD * 2 - THICK;
  const h = thumbOf(content.width, visible.width, scroll.left, hTrack);
  const v = thumbOf(content.height, visible.height, scroll.top, vTrack);
  return (
    <>
      {h && (
        <Bar
          horizontal
          thumb={h}
          track={hTrack}
          content={content.width}
          visible={visible.width}
          scroller={scroller}
          style={{
            left: insets.left + PAD,
            top: insets.top + visible.height - THICK - 2,
            width: hTrack,
          }}
        />
      )}
      {v && (
        <Bar
          horizontal={false}
          thumb={v}
          track={vTrack}
          content={content.height}
          visible={visible.height}
          scroller={scroller}
          style={{
            left: insets.left + visible.width - THICK - 2,
            top: insets.top + PAD,
            height: vTrack,
          }}
        />
      )}
    </>
  );
}

function Bar({
  horizontal,
  thumb,
  track,
  content,
  visible,
  scroller,
  style,
}: {
  horizontal: boolean;
  thumb: { start: number; length: number };
  track: number;
  content: number;
  visible: number;
  scroller: RefObject<HTMLDivElement>;
  style: CSSProperties;
}) {
  const [dragging, setDragging] = useState(false);
  const drag = useRef<{ at: number; from: number } | null>(null);
  const end = () => {
    drag.current = null;
    setDragging(false);
  };
  const scrollTo = (el: HTMLElement, value: number) => {
    if (horizontal) el.scrollLeft = value;
    else el.scrollTop = value;
  };
  return (
    // A pointer control: it adds nothing for screen readers or the keyboard.
    <div
      className={`canvas-bar ${horizontal ? "h" : "v"}${dragging ? " dragging" : ""}`}
      style={style}
      aria-hidden="true"
      onPointerDown={(e) => {
        e.stopPropagation();
        e.preventDefault();
        const el = scroller.current;
        if (!el || e.button !== 0) return;
        const r = e.currentTarget.getBoundingClientRect();
        const pos = horizontal ? e.clientX - r.left : e.clientY - r.top;
        const from = horizontal ? el.scrollLeft : el.scrollTop;
        if (pos < thumb.start || pos > thumb.start + thumb.length) {
          // A click on the track pages toward it.
          scrollTo(el, from + (pos < thumb.start ? -1 : 1) * visible * 0.9);
          return;
        }
        e.currentTarget.setPointerCapture(e.pointerId);
        drag.current = { at: horizontal ? e.clientX : e.clientY, from };
        setDragging(true);
      }}
      onPointerMove={(e) => {
        const d = drag.current;
        const el = scroller.current;
        if (!d || !el) return;
        const delta = (horizontal ? e.clientX : e.clientY) - d.at;
        scrollTo(el, d.from + scrollForDrag(content, visible, track, thumb.length, delta));
      }}
      onPointerUp={end}
      onPointerCancel={end}
    >
      <span
        style={
          horizontal
            ? { left: thumb.start, width: thumb.length }
            : { top: thumb.start, height: thumb.length }
        }
      />
    </div>
  );
}
