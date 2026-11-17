import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import { askOpen, openConfirmed, openQr, previewQr, splitUrl, useQr, type QrCode } from "./qrState";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const link: QrCode = { text: "https://example.com/a?b=1", rect: [0, 0, 10, 10], kind: "url" };

beforeEach(() => {
  vi.mocked(run).mockReset();
  useQr.setState({
    editing: null,
    check: null,
    tooLong: null,
    codes: null,
    confirmOpen: null,
    pill: null,
  });
});

describe("QR codes (SMT-036, SMT-038)", () => {
  it("shows the address with its domain apart", () => {
    expect(splitUrl("https://example.com/a?b=1")).toEqual(["https://", "example.com", "/a?b=1"]);
    expect(splitUrl("not a link")).toEqual(["", "not a link", ""]);
  });

  it("opens a link only after a yes", async () => {
    askOpen(link);
    await openConfirmed(false);
    expect(vi.mocked(run)).not.toHaveBeenCalled();
    askOpen(link);
    vi.mocked(run).mockResolvedValueOnce({ opened: true });
    await openConfirmed(true);
    expect(vi.mocked(run)).toHaveBeenCalledWith("smart.qr_open", { url: link.text });
  });

  it("says when the content is too long, with the most that fits", async () => {
    vi.mocked(run).mockRejectedValueOnce({ details: { bytes: 3000, max: 2331 } });
    await previewQr("x".repeat(3000), {
      kind: "text",
      ecc: "M",
      color: "#000000",
      background: "#FFFFFF",
      margin: 4,
    });
    expect(useQr.getState().tooLong).toEqual({ bytes: 3000, max: 2331 });
  });

  it("double click opens the panel with the code's content and style", () => {
    openQr({
      id: "o3",
      type: "qr",
      rect: { x: 0, y: 0, w: 100, h: 100 },
      rotation: 0,
      content: "https://example.com",
      style: { kind: "link", ecc: "H", color: "#112233", background: null, margin: 2 },
    });
    expect(useQr.getState().editing).toEqual({
      objectId: "o3",
      content: "https://example.com",
      style: { kind: "link", ecc: "H", color: "#112233", background: null, margin: 2 },
    });
  });
});
