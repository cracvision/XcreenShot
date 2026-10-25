// How each object looks on the canvas (EDT-024, EDT-026, EDT-029, EDT-040). Arrows
// and shapes are drawn by Konva with the shared geometry; text and blur are images
// Rust makes, so they look exactly as in exported files (ADR 0004).
import Konva from "konva";
import { useEffect, useRef } from "react";
import { Circle, Ellipse, Group, Image as KImage, Line, Rect } from "react-konva";
import { innerRect, pixelBounds } from "./boxes";
import { dash, type ArrowStyle, type Geometry } from "./geometry";
import type { BlurObject, ShapeObject, TextObject } from "./state";

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

export function ShapeShape({ o, scale }: { o: ShapeObject; scale: number }) {
  const { rect, style } = o;
  const ellipse = style.shapeType === "ellipse";
  const local = { x: -rect.w / 2, y: -rect.h / 2, w: rect.w, h: rect.h };
  const inner = innerRect(local, style.outlineWidth);
  const lineDash = dash(style.lineStyle, style.outlineWidth);
  return (
    <Group x={rect.x + rect.w / 2} y={rect.y + rect.h / 2} rotation={o.rotation} listening={false}>
      <Translucent opacity={style.opacity} scale={scale}>
        {style.fill && <Outline ellipse={ellipse} box={local} extra={{ fill: style.fill }} />}
        {style.outlineWidth > 0 && (
          <Outline
            ellipse={ellipse}
            box={inner}
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
  const w = image.naturalWidth;
  const h = image.naturalHeight;
  return (
    <KImage
      image={image}
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
