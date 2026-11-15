import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import { askSend, useLan } from "./lanState";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const status = (enabled: boolean) => ({
  enabled,
  running: enabled,
  name: "Ana",
  device: "PC",
  fingerprint: enabled ? "abc" : null,
  port: null,
  error: null,
});

beforeEach(() => {
  vi.mocked(run).mockReset();
  useLan.setState({ sending: null });
});

describe("sending on the local network (SHR-044)", () => {
  it("while it is off, Preferences > Share opens instead", async () => {
    vi.mocked(run).mockImplementation(async (id: string) =>
      id === "share.lan_status" ? status(false) : {},
    );
    await askSend(["a.xshot"]);
    expect(run).toHaveBeenCalledWith("settings.open", { section: "share" });
    expect(useLan.getState().sending).toBeNull();
  });

  it("on, the Send dialog opens with the files", async () => {
    vi.mocked(run).mockImplementation(async (id: string) => {
      if (id === "share.lan_status") return status(true);
      if (id === "share.lan_people") return { people: [] };
      if (id === "share.lan_pairings") return { pairings: [] };
      if (id === "share.lan_incoming") return { incoming: [] };
      if (id === "share.lan_sends") return { sends: [] };
      return {};
    });
    await askSend(["a.xshot", "b.png"]);
    expect(useLan.getState().sending).toEqual(["a.xshot", "b.png"]);
  });
});
