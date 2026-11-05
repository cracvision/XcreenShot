import { useEffect, useState } from "react";

/** Upper bound for the exit animation, in case animationend never fires. */
const EXIT_MS = 320;

/** Keeps a closing menu mounted until its exit animation has played. */
export function usePresence(open: boolean) {
  const [closing, setClosing] = useState(false);
  const [wasOpen, setWasOpen] = useState(open);
  if (open !== wasOpen) {
    setWasOpen(open);
    setClosing(!open);
  }
  useEffect(() => {
    if (!closing) return;
    const timer = window.setTimeout(() => setClosing(false), EXIT_MS);
    return () => window.clearTimeout(timer);
  }, [closing]);
  return {
    mounted: open || closing,
    state: open ? ("open" as const) : ("closed" as const),
    exited: () => setClosing(false),
  };
}
