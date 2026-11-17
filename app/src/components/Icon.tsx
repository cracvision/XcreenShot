// Interface glyphs: 16px grid, 1.75 stroke and round ends, like the viewfinder in the
// app icon. Decorative: the control that holds one carries the accessible name.
const PATHS = {
  chevron: "M4 6l4 4 4-4",
  check: "M3.5 8.5l3 3 6-7",
  lock: "M4.5 7.5h7v6h-7zM6 7.5v-2a2 2 0 0 1 4 0v2",
  alert: "M8 2.5l6 11H2zM8 7v2.5M8 11.8v.1",
  restore: "M3 8a5 5 0 1 0 1.5-3.6M3 2.5v2.5h2.5",
  close: "M4.5 4.5l7 7M11.5 4.5l-7 7",
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name, className }: { name: IconName; className?: string }) {
  return (
    <svg
      className={className ? `icon ${className}` : "icon"}
      width={16}
      height={16}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      <path d={PATHS[name]} />
    </svg>
  );
}
