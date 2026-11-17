// The managed configuration (APP-035): what the administrator fixed, read once per
// window; the core reads the file at start, so it does not change while the app runs.
import { useEffect, useState } from "react";
import { run } from "./commands";

export interface Managed {
  status: "inactive" | "active" | "errors";
  path: string;
  modified: string | null;
  errors: string[];
  defaults: Record<string, unknown>;
  locked: Record<string, unknown>;
  policy: {
    disableVideoCapture?: boolean;
    hideExperimental?: boolean;
    disabledDestinations?: string[];
    feedback?: string;
    [k: string]: unknown;
  };
}

let cached: Promise<Managed> | null = null;
/** What was read, for menus built outside React. */
let known: Managed | null = null;

export function managedNow(): Managed | null {
  void loadManaged();
  return known;
}

export function loadManaged(): Promise<Managed> {
  cached ??= run<Managed>("app.managed").catch(() => ({
    status: "inactive" as const,
    path: "",
    modified: null,
    errors: [],
    defaults: {},
    locked: {},
    policy: {},
  }));
  void cached.then((m) => (known = m));
  return cached;
}

export function useManaged(): Managed | null {
  const [m, setM] = useState<Managed | null>(null);
  useEffect(() => {
    let alive = true;
    void loadManaged().then((v) => alive && setM(v));
    return () => {
      alive = false;
    };
  }, []);
  return m;
}

/** A sharing destination the administrator turned off (APP-035). */
export function destinationOff(m: Managed | null, id: string): boolean {
  return !!m?.policy.disabledDestinations?.includes(id);
}
