import { describe, expect, it } from "vitest";
import { destinationOff, type Managed } from "./managed";

const m = (policy: Managed["policy"]): Managed => ({
  status: "active",
  path: "",
  modified: null,
  errors: [],
  defaults: {},
  locked: {},
  policy,
});

describe("managed configuration (APP-035)", () => {
  it("hides the destinations the administrator turned off", () => {
    expect(destinationOff(m({ disabledDestinations: ["lan"] }), "lan")).toBe(true);
    expect(destinationOff(m({ disabledDestinations: ["lan"] }), "file")).toBe(false);
    expect(destinationOff(null, "lan")).toBe(false);
  });
});
