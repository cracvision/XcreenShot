// The camera in the recording bar (VID-011, VID-031 to VID-036): a switch, its menu
// (which camera, where, how big, which shape, mirror and border) and Switch view.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Menu, type MenuItem } from "../components/Menu";
import { run } from "../lib/commands";
import { settingValue, useApp, useSetting, useStringSetting } from "../store";
import type { SoundLevels } from "./recSound";

const camera = (p: Record<string, unknown>) => void run("record.camera", p).catch(() => undefined);

/** The sizes of the menu (VID-032). */
const SIZES: [string, number][] = [
  ["small", 0.15],
  ["medium", 0.25],
  ["large", 0.4],
];
const CORNERS = ["topLeft", "topRight", "bottomLeft", "bottomRight"];
const SHAPES = ["circle", "roundedRect", "rect", "square", "roundedSquare"];
const BORDERS = ["none", "solid", "gradient"];

interface Cam {
  id: string;
  name: string;
}

export function CameraControl({
  levels,
  up,
  onMenu,
}: {
  levels: SoundLevels | null;
  up: boolean;
  /** The menu appears with its number of entries, or goes (0). */
  onMenu: (rows: number) => void;
}) {
  const { t } = useTranslation();
  const chosen = useStringSetting("video.cameraDevice", "");
  const placement = useStringSetting("video.cameraPlacement", "pip");
  const corner = useStringSetting("video.cameraCorner", "bottomRight");
  const shape = useStringSetting("video.cameraShape", "roundedRect");
  const aspect = useStringSetting("video.cameraAspect", "16:9");
  const border = useStringSetting("video.cameraBorder", "none");
  const mirror = useSetting("video.cameraMirror", true);
  const size = useApp((s) => {
    const v = settingValue(s.settings, "video.cameraSize");
    return typeof v === "number" ? v : 0.25;
  });
  const [cams, setCams] = useState<Cam[] | null>(null);
  const [asked, setAsked] = useState(0);
  // The cameras, again each time the menu opens (VID-043).
  useEffect(() => {
    let alive = true;
    void run<{ cameras: Cam[] }>("record.cameras")
      .then((r) => {
        if (alive) setCams(r.cameras);
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [asked]);
  const c = levels?.camera;
  if (!c || !c.enabled) return null;
  const on = c.on && c.available;
  const recording = levels.status === "recording";
  const locked = recording ? { title: t("ui.recbar.camera_locked") } : {};
  const choice = (
    id: string,
    label: string,
    checked: boolean,
    params: Record<string, unknown>,
  ): MenuItem => ({
    id,
    label,
    checked,
    onSelect: () => camera(params),
  });
  const option = (setting: string, value: string) => t(`settings.${setting}.options.${value}`);
  const rect = shape === "rect" || shape === "roundedRect";
  const items: MenuItem[] = [
    {
      id: "default",
      label: t("ui.recbar.camera_default"),
      checked: chosen === "",
      disabled: recording,
      ...locked,
      onSelect: () => camera({ on: true, device: "" }),
    },
    ...(cams ?? []).map((cam): MenuItem => ({
      id: cam.id,
      label: cam.name,
      checked: chosen === cam.id,
      disabled: recording,
      ...locked,
      onSelect: () => camera({ on: true, device: cam.id }),
    })),
    {
      ...choice("pip", t("ui.recbar.camera_pip"), placement === "pip", { placement: "pip" }),
      separator: true,
    },
    choice("fullscreen", t("ui.recbar.camera_fullscreen"), placement === "fullscreen", {
      placement: "fullscreen",
    }),
    {
      id: "corner",
      label: t("ui.recbar.camera_corner"),
      separator: true,
      onSelect: () => undefined,
      submenu: CORNERS.map((k) =>
        choice(k, option("video_cameraCorner", k), corner === k, { corner: k, placement: "pip" }),
      ),
    },
    {
      id: "size",
      label: t("ui.recbar.camera_size"),
      onSelect: () => undefined,
      submenu: SIZES.map(([k, v]) =>
        choice(k, t(`ui.recbar.size_${k}`), Math.abs(size - v) < 0.02, { size: v }),
      ),
    },
    {
      id: "shape",
      label: t("ui.recbar.camera_shape"),
      onSelect: () => undefined,
      submenu: [
        ...SHAPES.map((k) => choice(k, option("video_cameraShape", k), shape === k, { shape: k })),
        {
          ...choice("16:9", "16:9", aspect === "16:9", { aspect: "16:9" }),
          separator: true,
          disabled: !rect,
        },
        { ...choice("4:3", "4:3", aspect === "4:3", { aspect: "4:3" }), disabled: !rect },
      ],
    },
    {
      id: "border",
      label: t("ui.recbar.camera_border"),
      onSelect: () => undefined,
      submenu: BORDERS.map((k) =>
        choice(k, option("video_cameraBorder", k), border === k, { border: k }),
      ),
    },
    choice("mirror", t("ui.recbar.camera_mirror"), mirror, { mirror: !mirror }),
  ];
  return (
    <span className="recbar-split">
      <button
        type="button"
        className="btn small recbar-toggle"
        aria-pressed={on}
        disabled={!c.available}
        title={
          !c.available
            ? t("ui.recbar.camera_none")
            : c.device
              ? `${t("ui.recbar.camera")}: ${c.device}`
              : t("ui.recbar.camera")
        }
        onClick={() => camera({ on: !on })}
      >
        {t("ui.recbar.camera")}
      </button>
      {c.available && (
        <Menu
          label={null}
          items={items}
          align="end"
          side={up ? "top" : "bottom"}
          triggerClass="btn small icon recbar-chev"
          ariaLabel={t("ui.recbar.camera_options")}
          title={t("ui.recbar.camera_options")}
          onShown={(shown) => {
            if (shown) setAsked((n) => n + 1);
            onMenu(shown ? items.length : 0);
          }}
        />
      )}
      {on && (
        <button
          type="button"
          className="btn icon small"
          aria-label={t("ui.recbar.camera_swap")}
          title={t("ui.recbar.camera_swap")}
          onClick={() => void run("record.camera_swap").catch(() => undefined)}
        >
          ⇄
        </button>
      )}
    </span>
  );
}
