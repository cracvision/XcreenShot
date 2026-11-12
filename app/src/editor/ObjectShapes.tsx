// How each object looks on the canvas (EDT-024, EDT-026, EDT-029, EDT-040). Arrows
// and shapes are drawn by Konva with the shared geometry; text and blur are images
// Rust makes, so they look exactly as in exported files (ADR 0004).
import Konva from "konva";
import { useEffect, useRef } from "react";
import { Circle, Ellipse, Group, Image as KImage, Line, Rect } from "react-konva";
import { innerRect, pixelBounds } from "./boxes";
import { lensOf } from "./state";
import { dash, type ArrowStyle, type Geometry } from "./geometry";
import type {
  BlurObject,
  CalloutObject,
  Fill,
  HighlightObject,
  ImageObject,
  MagnifyObject,
  PenObject,
  ShapeObject,
  SpotlightObject,
  StampObject,
  StepObject,
  TextObject,
  ZoneObject,
} from "./state";
import { placement } from "./templateUi";

/** Draws its children whole and blends them once when translucent, like the export. */
function Translucent({
  opacity,
  scale,
  children,
}: {
  opacity: number;
  scale: number;
  children: React.ReactNode;
}) {
  const ref = useRef<Konva.Group>(null);
  const translucent = opacity < 100;
  useEffect(() => {
    const node = ref.current;
    if (!node) return;
    if (translucent) node.cache({ pixelRatio: Math.max(1, scale * window.devicePixelRatio) });
    else node.clearCache();
  });
  return (
    <Group ref={ref} opacity={opacity / 100} listening={false}>
      {children}
    </Group>
  );
}

export function ArrowShape({ g, style, scale }: { g: Geometry; style: ArrowStyle; scale: number }) {
  const color = style.color;
  return (
    <Translucent opacity={style.opacity} scale={scale}>
      {g.shaft && (
        <Line
          points={g.shaft}
          stroke={color}
          strokeWidth={g.width}
          lineCap="butt"
          lineJoin="round"
          {...(g.dash ? { dash: g.dash } : {})}
        />
      )}
      {g.caps.map((s, i) =>
        s.kind === "circle" ? (
          <Circle
            key={i}
            x={s.cx}
            y={s.cy}
            radius={s.r}
            {...(s.fill ? { fill: color } : { stroke: color, strokeWidth: s.strokeWidth })}
          />
        ) : (
          <Line
            key={i}
            points={s.points}
            closed={s.closed}
            lineCap="butt"
            lineJoin="round"
            {...(s.fill
              ? { fill: color, strokeEnabled: false }
              : { stroke: color, strokeWidth: s.strokeWidth })}
          />
        ),
      )}
    </Translucent>
  );
}

/** A rectangle or ellipse filling `w` × `h` from (`x`, `y`), turned around the box center. */
function Outline(props: {
  ellipse: boolean;
  box: { x: number; y: number; w: number; h: number };
  extra: Konva.ShapeConfig;
}) {
  const { ellipse, box, extra } = props;
  return ellipse ? (
    <Ellipse
      x={box.x + box.w / 2}
      y={box.y + box.h / 2}
      radiusX={Math.max(box.w / 2, 0)}
      radiusY={Math.max(box.h / 2, 0)}
      {...extra}
    />
  ) : (
    <Rect x={box.x} y={box.y} width={Math.max(box.w, 0)} height={Math.max(box.h, 0)} {...extra} />
  );
}

/** The corners of a triangle, a diamond or a polygon filling `box`, or null for the
 * shapes Konva draws itself (EDT-030). The same as `shape_path` in Rust. */
function shapePoints(o: ShapeObject, box: { x: number; y: number; w: number; h: number }) {
  const { x, y, w, h } = box;
  switch (o.style.shapeType) {
    case "triangle":
      return [x + w / 2, y, x + w, y + h, x, y + h];
    case "diamond":
      return [x + w / 2, y, x + w, y + h / 2, x + w / 2, y + h, x, y + h / 2];
    case "polygon":
      return o.points && o.points.length >= 3
        ? o.points.flatMap((p) => [x + p[0] * w, y + p[1] * h])
        : null;
    default:
      return null;
  }
}

/** One of a shape's two layers: its fill over `box`, or its outline along it. */
/** Konva fill settings for a color or a gradient over `box` (EDT-020), for a node
 * whose own origin is at `origin`. The same geometry as `Gradient::geometry` in Rust. */
function fillProps(
  fill: Fill,
  box: { x: number; y: number; w: number; h: number },
  origin: [number, number],
): Konva.ShapeConfig {
  if (typeof fill === "string") return { fill };
  const stops = [...fill.stops]
    .sort((a, b) => a.pos - b.pos)
    .flatMap((s) => [s.pos / 100, s.color]);
  const cx = box.x + box.w / 2 - origin[0];
  const cy = box.y + box.h / 2 - origin[1];
  if (fill.type === "radial")
    return {
      fillRadialGradientStartPoint: { x: cx, y: cy },
      fillRadialGradientEndPoint: { x: cx, y: cy },
      fillRadialGradientStartRadius: 0,
      fillRadialGradientEndRadius: Math.max(box.w, box.h) / 2,
      fillRadialGradientColorStops: stops,
    };
  const a = (fill.angle * Math.PI) / 180;
  const dx = Math.cos(a);
  const dy = Math.sin(a);
  const half = (box.w * Math.abs(dx) + box.h * Math.abs(dy)) / 2;
  return {
    fillLinearGradientStartPoint: { x: cx - dx * half, y: cy - dy * half },
    fillLinearGradientEndPoint: { x: cx + dx * half, y: cy + dy * half },
    fillLinearGradientColorStops: stops,
  };
}

function ShapeLayer(props: {
  o: ShapeObject;
  box: { x: number; y: number; w: number; h: number };
  radius: number;
  extra: Konva.ShapeConfig;
  paint?: Fill;
}) {
  const { o, box, radius, extra, paint } = props;
  const pts = shapePoints(o, box);
  // Each node has its own origin; the gradient is placed in it.
  const fillAt = (origin: [number, number]) => (paint ? fillProps(paint, box, origin) : {});
  if (pts) return <Line points={pts} closed {...extra} {...fillAt([0, 0])} />;
  if (o.style.shapeType === "roundedRectangle")
    return (
      <Rect
        x={box.x}
        y={box.y}
        width={Math.max(box.w, 0)}
        height={Math.max(box.h, 0)}
        cornerRadius={Math.max(0, Math.min(radius, box.w / 2, box.h / 2))}
        {...extra}
        {...fillAt([box.x, box.y])}
      />
    );
  const ellipse = o.style.shapeType === "ellipse";
  return (
    <Outline
      ellipse={ellipse}
      box={box}
      extra={{
        ...extra,
        ...fillAt(ellipse ? [box.x + box.w / 2, box.y + box.h / 2] : [box.x, box.y]),
      }}
    />
  );
}

export function ShapeShape({ o, scale }: { o: ShapeObject; scale: number }) {
  const { rect, style } = o;
  const local = { x: -rect.w / 2, y: -rect.h / 2, w: rect.w, h: rect.h };
  const inner = innerRect(local, style.outlineWidth);
  const lineDash = dash(style.lineStyle, style.outlineWidth);
  const radius = style.cornerRadius ?? 0;
  return (
    <Group x={rect.x + rect.w / 2} y={rect.y + rect.h / 2} rotation={o.rotation} listening={false}>
      <Translucent opacity={style.opacity} scale={scale}>
        {style.fill && (
          <ShapeLayer o={o} box={local} radius={radius} extra={{}} paint={style.fill} />
        )}
        {style.outlineWidth > 0 && (
          <ShapeLayer
            o={o}
            box={inner}
            radius={Math.max(0, radius - style.outlineWidth / 2)}
            extra={{
              stroke: style.outline,
              strokeWidth: style.outlineWidth,
              lineJoin: "miter",
              lineCap: "butt",
              ...(lineDash ? { dash: lineDash } : {}),
            }}
          />
        )}
      </Translucent>
    </Group>
  );
}

export function TextShape({ o, image }: { o: TextObject; image: HTMLImageElement | undefined }) {
  if (!image) return null;
  // Its picture at its own size; stretched only while several are resized (EDT-080).
  const off = (a: number, b: number) => Math.abs(a - b) > 1;
  const stretched =
    off(o.layout.width, image.naturalWidth) || off(o.layout.height, image.naturalHeight);
  const w = stretched ? o.layout.width : image.naturalWidth;
  const h = stretched ? o.layout.height : image.naturalHeight;
  return (
    <KImage
      image={image}
      width={w}
      height={h}
      x={o.at[0] + w / 2}
      y={o.at[1] + h / 2}
      offsetX={w / 2}
      offsetY={h / 2}
      rotation={o.rotation}
      opacity={o.style.opacity / 100}
      listening={false}
    />
  );
}

/** A floating image (EDT-078): its picture stretched to its box and turned. */
export function ImageShape({ o, image }: { o: ImageObject; image: HTMLImageElement | undefined }) {
  if (!image) return null;
  const { x, y, w, h } = o.rect;
  return (
    <KImage
      image={image}
      x={x + w / 2}
      y={y + h / 2}
      width={w}
      height={h}
      offsetX={w / 2}
      offsetY={h / 2}
      rotation={o.rotation}
      opacity={o.style.opacity / 100}
      listening={false}
    />
  );
}

/** A template's zone (EDT-098): its picture cropped to it, at its framing (`offset`
 * while it is being moved); an empty one, a dashed placeholder the editor shows. */
export function ZoneShape(props: {
  o: ZoneObject;
  image: HTMLImageElement | undefined;
  offset?: [number, number] | undefined;
  scale: number;
  dim?: boolean;
}) {
  const { o, image, offset, scale, dim } = props;
  const r = o.rect;
  if (!o.image) {
    return (
      <Rect
        x={r.x}
        y={r.y}
        width={r.w}
        height={r.h}
        cornerRadius={6 / scale}
        fill="rgba(0, 100, 251, 0.06)"
        stroke="rgba(0, 100, 251, 0.55)"
        strokeWidth={1.5 / scale}
        dash={[8 / scale, 5 / scale]}
        listening={false}
      />
    );
  }
  if (!image) return null;
  const size = { w: image.naturalWidth || image.width, h: image.naturalHeight || image.height };
  const { x, y, k } = placement(o, size, offset);
  return (
    <Group clipX={r.x} clipY={r.y} clipWidth={r.w} clipHeight={r.h} listening={false}>
      <KImage
        image={image}
        x={x}
        y={y}
        width={size.w * k}
        height={size.h * k}
        opacity={dim ? 0.45 : 1}
      />
    </Group>
  );
}

export function BlurShape(props: {
  o: BlurObject;
  image: HTMLImageElement | undefined;
  /** Where the committed object is, to show its image moved during a drag. */
  committed: BlurObject;
  canvas: { width: number; height: number };
}) {
  const { o, image, committed, canvas } = props;
  const { rect, style } = o;
  if (style.blurType === "solid") {
    return (
      <Group
        x={rect.x + rect.w / 2}
        y={rect.y + rect.h / 2}
        rotation={o.rotation}
        listening={false}
      >
        <Outline
          ellipse={style.shapeType === "ellipse"}
          box={{ x: -rect.w / 2, y: -rect.h / 2, w: rect.w, h: rect.h }}
          extra={{ fill: style.color }}
        />
      </Group>
    );
  }
  const bounds = pixelBounds(committed.rect, committed.rotation, canvas.width, canvas.height);
  if (!image || !bounds) return null;
  const same =
    committed.rect.w === rect.w && committed.rect.h === rect.h && committed.rotation === o.rotation;
  if (!same) {
    // Being resized or turned: an outline until the new blur is ready.
    return (
      <Group
        x={rect.x + rect.w / 2}
        y={rect.y + rect.h / 2}
        rotation={o.rotation}
        listening={false}
      >
        <Outline
          ellipse={style.shapeType === "ellipse"}
          box={{ x: -rect.w / 2, y: -rect.h / 2, w: rect.w, h: rect.h }}
          extra={{ fill: "rgba(128,128,128,0.5)" }}
        />
      </Group>
    );
  }
  return (
    <KImage
      image={image}
      x={bounds[0] + rect.x - committed.rect.x}
      y={bounds[1] + rect.y - committed.rect.y}
      listening={false}
    />
  );
}

/** A step marker as Rust draws it (EDT-035), centered on its center; while it is
 * resized or turned, the picture follows until the new one is ready. */
export function StepShape(props: {
  o: StepObject;
  committed: StepObject;
  image: HTMLImageElement | undefined;
}) {
  const { o, committed, image } = props;
  if (!image) return null;
  const k = committed.rect.w > 0 ? o.rect.w / committed.rect.w : 1;
  return (
    <KImage
      image={image}
      x={o.rect.x + o.rect.w / 2}
      y={o.rect.y + o.rect.h / 2}
      offsetX={image.naturalWidth / 2}
      offsetY={image.naturalHeight / 2}
      scaleX={k}
      scaleY={k}
      rotation={o.rotation - committed.rotation}
      opacity={1}
      listening={false}
    />
  );
}

/** A callout as Rust draws it (EDT-028), centered on its body's center. While the body
 * is resized or turned, the picture follows until the new one is ready; while a tail is
 * dragged, the canvas draws that tail on top. */
export function CalloutShape(props: {
  o: CalloutObject;
  committed: CalloutObject;
  image: HTMLImageElement | undefined;
}) {
  const { o, committed, image } = props;
  if (!image) return null;
  const kx = committed.rect.w > 0 ? o.rect.w / committed.rect.w : 1;
  const ky = committed.rect.h > 0 ? o.rect.h / committed.rect.h : 1;
  return (
    <KImage
      image={image}
      x={o.rect.x + o.rect.w / 2}
      y={o.rect.y + o.rect.h / 2}
      offsetX={image.naturalWidth / 2}
      offsetY={image.naturalHeight / 2}
      scaleX={kx}
      scaleY={ky}
      rotation={o.rotation - committed.rotation}
      listening={false}
    />
  );
}

/** A highlight (EDT-036): its color multiplied into what is below, like a marker. */
export function HighlightShape({ o }: { o: HighlightObject }) {
  const { rect, style } = o;
  const common = {
    fill: style.color,
    opacity: style.opacity / 100,
    globalCompositeOperation: "multiply" as const,
    listening: false,
  };
  return (
    <Group x={rect.x + rect.w / 2} y={rect.y + rect.h / 2} rotation={o.rotation} listening={false}>
      {style.shape === "ellipse" ? (
        <Ellipse radiusX={rect.w / 2} radiusY={rect.h / 2} {...common} />
      ) : (
        <Rect
          x={-rect.w / 2}
          y={-rect.h / 2}
          width={rect.w}
          height={rect.h}
          cornerRadius={style.shape === "roundedRectangle" ? 0.25 * Math.min(rect.w, rect.h) : 0}
          {...common}
        />
      )}
    </Group>
  );
}

/** The middle of a stroke's points: where Rust centers its picture. */
function middle(points: [number, number][]): [number, number] {
  const xs = points.map((p) => p[0]);
  const ys = points.map((p) => p[1]);
  return [(Math.min(...xs) + Math.max(...xs)) / 2, (Math.min(...ys) + Math.max(...ys)) / 2];
}

/** A stroke as Rust draws it (EDT-037), centered on the middle of its points; while it
 * is moved or stretched, the picture follows until the new one is ready. */
export function PenShape(props: {
  o: PenObject;
  committed: PenObject;
  image: HTMLImageElement | undefined;
}) {
  const { o, committed, image } = props;
  if (!image || o.points.length === 0) return null;
  const [cx, cy] = middle(o.points);
  const span = (pts: [number, number][], i: 0 | 1) =>
    Math.max(...pts.map((p) => p[i])) - Math.min(...pts.map((p) => p[i]));
  const k = (i: 0 | 1) => {
    const before = span(committed.points, i);
    return before > 0 ? span(o.points, i) / before : 1;
  };
  return (
    <KImage
      image={image}
      x={cx}
      y={cy}
      offsetX={image.naturalWidth / 2}
      offsetY={image.naturalHeight / 2}
      scaleX={k(0)}
      scaleY={k(1)}
      listening={false}
    />
  );
}

/** The stroke being drawn (EDT-037), before Rust smooths and draws it. */
export function PenPreview(props: {
  points: [number, number][];
  color: string;
  width: number;
  opacity: number;
  square: boolean;
}) {
  const { points, color, width, opacity, square } = props;
  return (
    <Line
      points={points.flat()}
      stroke={color}
      strokeWidth={width}
      opacity={opacity / 100}
      lineCap={square ? "square" : "round"}
      lineJoin={square ? "miter" : "round"}
      tension={0.3}
      listening={false}
    />
  );
}

/** A lens as Rust draws it (EDT-038), centered on the middle of its zone and lens.
 * While it changes, outlines show where the zone and the lens go. */
export function MagnifyShape(props: {
  o: MagnifyObject;
  committed: MagnifyObject;
  image: HTMLImageElement | undefined;
}) {
  const { o, committed, image } = props;
  const lens = lensOf(o);
  const same =
    JSON.stringify([o.source, o.at, o.style]) ===
    JSON.stringify([committed.source, committed.at, committed.style]);
  const span = (m: MagnifyObject) => {
    const l = lensOf(m);
    const x0 = Math.min(m.source.x, l.x);
    const y0 = Math.min(m.source.y, l.y);
    const x1 = Math.max(m.source.x + m.source.w, l.x + l.w);
    const y1 = Math.max(m.source.y + m.source.h, l.y + l.h);
    return [(x0 + x1) / 2, (y0 + y1) / 2] as const;
  };
  if (!same || !image) {
    const round = o.style.shape === "circle";
    const box = (r: { x: number; y: number; w: number; h: number }, key: string) =>
      round ? (
        <Ellipse
          key={key}
          x={r.x + r.w / 2}
          y={r.y + r.h / 2}
          radiusX={r.w / 2}
          radiusY={r.h / 2}
          stroke={o.style.outline}
          strokeWidth={Math.max(1, o.style.outlineWidth)}
          dash={[6, 4]}
          listening={false}
        />
      ) : (
        <Rect
          key={key}
          x={r.x}
          y={r.y}
          width={r.w}
          height={r.h}
          stroke={o.style.outline}
          strokeWidth={Math.max(1, o.style.outlineWidth)}
          dash={[6, 4]}
          listening={false}
        />
      );
    return (
      <>
        {box(o.source, "zone")}
        {box(lens, "lens")}
      </>
    );
  }
  const [cx, cy] = span(o);
  return (
    <KImage
      image={image}
      x={cx}
      y={cy}
      offsetX={image.naturalWidth / 2}
      offsetY={image.naturalHeight / 2}
      listening={false}
    />
  );
}

/** Spotlights (EDT-039): the topmost one's picture is the whole canvas with the outside
 * changed; the others draw nothing of their own. While one changes, its zone shows. */
export function SpotlightShape(props: {
  o: SpotlightObject;
  committed: SpotlightObject;
  image: HTMLImageElement | undefined;
}) {
  const { o, committed, image } = props;
  const same =
    JSON.stringify([o.rect, o.rotation, o.style]) ===
    JSON.stringify([committed.rect, committed.rotation, committed.style]);
  if (!same)
    return (
      <Group
        x={o.rect.x + o.rect.w / 2}
        y={o.rect.y + o.rect.h / 2}
        rotation={o.rotation}
        listening={false}
      >
        <Outline
          ellipse={o.style.shape === "ellipse"}
          box={{ x: -o.rect.w / 2, y: -o.rect.h / 2, w: o.rect.w, h: o.rect.h }}
          extra={{ fill: "rgba(255,255,255,0.15)" }}
        />
      </Group>
    );
  if (!image) return null;
  return <KImage image={image} x={0} y={0} listening={false} />;
}

/** A stamp as Rust draws it (EDT-031), centered on its box; while it is resized or
 * turned, the picture follows until the new one is ready. */
export function StampShape(props: {
  o: Pick<StampObject, "rect" | "rotation">;
  committed: Pick<StampObject, "rect" | "rotation">;
  image: HTMLImageElement | undefined;
}) {
  const { o, committed, image } = props;
  if (!image) return null;
  const kx = committed.rect.w > 0 ? o.rect.w / committed.rect.w : 1;
  const ky = committed.rect.h > 0 ? o.rect.h / committed.rect.h : 1;
  return (
    <KImage
      image={image}
      x={o.rect.x + o.rect.w / 2}
      y={o.rect.y + o.rect.h / 2}
      offsetX={image.naturalWidth / 2}
      offsetY={image.naturalHeight / 2}
      scaleX={kx}
      scaleY={ky}
      rotation={o.rotation - committed.rotation}
      listening={false}
    />
  );
}
