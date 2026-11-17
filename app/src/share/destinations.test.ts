import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import { destinationItems, shareTo, useShare } from "./destinations";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const t = (k: string) => k;

beforeEach(() => {
  vi.mocked(run).mockReset();
  vi.mocked(run).mockResolvedValue({ destinations: [] });
  useShare.setState({
    large: null,
    dests: [
      { id: "email", name: "Email", available: true },
      { id: "word", name: "Word", available: false },
      { id: "app", name: "App", available: true },
    ],
  });
});

describe("sharing destinations (SHR-020 to SHR-026)", () => {
  it("lists only the ones that can be used, with Another app as a submenu", () => {
    const items = destinationItems(() => ({ documentId: "d1" }), t);
    expect(items.map((i) => i.id)).toEqual(["dest_email", "dest_app"]);
    expect(items[1]?.submenu?.at(-1)?.id).toBe("app_pick");
  });

  it("email over 20 MB asks before opening", async () => {
    vi.mocked(run).mockResolvedValue({ status: "too_large", size: 30_000_000 });
    await shareTo("email", { paths: ["a.mp4"] });
    expect(useShare.getState().large).toEqual({ target: { paths: ["a.mp4"] }, size: 30_000_000 });
  });
});
