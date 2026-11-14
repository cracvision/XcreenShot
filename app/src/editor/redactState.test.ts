import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import {
  closeRedact,
  softRedactions,
  stepRedaction,
  termName,
  toggleCategory,
  useRedact,
  type RedactResult,
} from "./redactState";
import { useEditor, type DocDetails } from "./state";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const items: RedactResult["items"] = [
  { objectId: "o1", category: "email", rect: [10, 10, 100, 20] },
  { objectId: "o2", category: "face", rect: [200, 300, 80, 80] },
];

const result = (r: Partial<RedactResult> = {}): RedactResult => ({
  documentId: "d1",
  redactions: items.length,
  counts: { email: 1, face: 1 },
  categories: ["email", "phone", "face"],
  invalidPatterns: [],
  textRead: true,
  items,
  ...r,
});

beforeEach(() => {
  vi.mocked(run).mockReset();
  closeRedact();
  useEditor.setState({ selection: [], views: { d1: { zoom: 200, cx: 0, cy: 0 } } });
});

describe("auto redact panel (SMT-010 to SMT-013)", () => {
  it("Next and Previous select each redaction and center the view on it, round", () => {
    useRedact.setState({ documentId: "d1", result: result() });
    stepRedaction(1);
    expect(useEditor.getState().selection).toEqual(["o1"]);
    expect(useEditor.getState().views.d1).toEqual({ zoom: 200, cx: 60, cy: 20 });
    stepRedaction(1);
    expect(useEditor.getState().selection).toEqual(["o2"]);
    expect(useEditor.getState().views.d1).toMatchObject({ cx: 240, cy: 340 });
    stepRedaction(1);
    expect(useEditor.getState().selection).toEqual(["o1"]);
    stepRedaction(-1);
    expect(useEditor.getState().selection).toEqual(["o2"]);
  });

  it("a category switched on keeps the panel's order; off, it leaves the list", async () => {
    vi.mocked(run).mockResolvedValue(result({ categories: ["email", "card", "phone", "face"] }));
    useRedact.setState({ documentId: "d1", categories: ["email", "phone", "face"] });
    await toggleCategory("card");
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.redact", {
      documentId: "d1",
      categories: ["email", "phone", "card", "face"],
    });
    useRedact.setState({ categories: ["email", "phone", "card", "face"] });
    await toggleCategory("phone");
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.redact", {
      documentId: "d1",
      categories: ["email", "card", "face"],
    });
  });

  it("a term counts under its name, else its pattern or words", () => {
    expect(termName({ name: " Pedidos ", pattern: "PO-\\d{6}" })).toBe("Pedidos");
    expect(termName({ pattern: "PO-\\d{6}" })).toBe("PO-\\d{6}");
    expect(termName({ text: "Proyecto Orión" })).toBe("Proyecto Orión");
  });

  it("Done reminds only of the managed redactions that blur or pixelate", () => {
    useRedact.setState({ documentId: "d1", result: result() });
    const blur = (id: string, blurType: "blur" | "pixelate" | "solid") => ({
      id,
      type: "blur" as const,
      rect: { x: 0, y: 0, w: 10, h: 10 },
      rotation: 0,
      style: { blurType, color: "#000000", intensity: 60, shapeType: "rectangle" as const },
    });
    useEditor.setState({
      doc: {
        documentId: "d1",
        objects: [blur("o1", "solid"), blur("o2", "pixelate"), blur("o9", "blur")],
      } as unknown as DocDetails,
    });
    expect(softRedactions()).toBe(1);
  });
});
