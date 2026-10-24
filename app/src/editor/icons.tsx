// XcreenShot's own toolbar icons (SPEC §2.4): simple 20 px line drawings.
import type { ReactNode } from "react";

function Svg({ children }: { children: ReactNode }) {
  return (
    <svg
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {children}
    </svg>
  );
}

const ICONS = {
  capture: (
    <Svg>
      <path d="M3 7V4h3M14 4h3v3M17 13v3h-3M6 16H3v-3" />
      <circle cx="10" cy="10" r="2.5" />
    </Svg>
  ),
  move: (
    <Svg>
      <path d="M5 3l9 6-4 1.2L12.5 16l-2 1-2.4-5.6L5 14z" />
    </Svg>
  ),
  arrow: (
    <Svg>
      <path d="M4 16L15 5" />
      <path d="M8.5 4.5H15.5V11.5" />
    </Svg>
  ),
  undo: (
    <Svg>
      <path d="M7 5L3.5 8.5 7 12" />
      <path d="M4 8.5h7.5a4.5 4.5 0 010 9H9" />
    </Svg>
  ),
  redo: (
    <Svg>
      <path d="M13 5l3.5 3.5L13 12" />
      <path d="M16 8.5H8.5a4.5 4.5 0 000 9H11" />
    </Svg>
  ),
  open: (
    <Svg>
      <path d="M2.5 15.5V5a1 1 0 011-1h4l1.5 2h7a1 1 0 011 1v1.5" />
      <path d="M2.5 15.5l2.5-6.5h13l-2.5 6.5z" />
    </Svg>
  ),
  save: (
    <Svg>
      <path d="M4 3.5h9.5L16.5 6.5V16a.5.5 0 01-.5.5H4a.5.5 0 01-.5-.5V4a.5.5 0 01.5-.5z" />
      <path d="M6.5 3.5v4h6v-4M6.5 16.5v-5h7v5" />
    </Svg>
  ),
  copy: (
    <Svg>
      <rect x="6.5" y="6.5" width="10" height="10" rx="1.5" />
      <path d="M13.5 4.5v-.5A1 1 0 0012.5 3H4a1 1 0 00-1 1v8.5a1 1 0 001 1h.5" />
    </Svg>
  ),
  share: (
    <Svg>
      <path d="M10 12.5V3M6.5 6.5L10 3l3.5 3.5" />
      <path d="M4 10.5V16a1 1 0 001 1h10a1 1 0 001-1v-5.5" />
    </Svg>
  ),
  settings: (
    <Svg>
      <circle cx="10" cy="10" r="2.5" />
      <path d="M10 2.5v2M10 15.5v2M17.5 10h-2M4.5 10h-2M15.3 4.7l-1.4 1.4M6.1 13.9l-1.4 1.4M15.3 15.3l-1.4-1.4M6.1 6.1L4.7 4.7" />
    </Svg>
  ),
  close: (
    <Svg>
      <path d="M5.5 5.5l9 9M14.5 5.5l-9 9" />
    </Svg>
  ),
  minus: (
    <Svg>
      <path d="M5 10h10" />
    </Svg>
  ),
  plus: (
    <Svg>
      <path d="M5 10h10M10 5v10" />
    </Svg>
  ),
  chevron: (
    <Svg>
      <path d="M6 8l4 4 4-4" />
    </Svg>
  ),
};

export type IconName = keyof typeof ICONS;

export function Icon({ name }: { name: IconName }) {
  return ICONS[name];
}
