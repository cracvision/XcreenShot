// Effects on capture seen from the editor (CAP-051): which of the document's effects
// the capture panel adds to every new capture, and the panel's list without some.

/** The effect types the capture panel adds to every capture. */
export function captureTypes(effects: readonly { type: string }[]): Set<string> {
  return new Set(effects.map((e) => e.type));
}

/** The panel's list without the given types, the rest in their order. */
export function withoutTypes<T extends { type: string }>(
  effects: readonly T[],
  types: readonly string[],
): T[] {
  return effects.filter((e) => !types.includes(e.type));
}
