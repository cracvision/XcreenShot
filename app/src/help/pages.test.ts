import { describe, expect, it } from "vitest";
import { blocks, helpPage, pieces, TOPICS } from "./pages";

describe("help pages (APP-015)", () => {
  it("exist in English and Spanish for every topic", () => {
    for (const id of TOPICS) {
      const en = helpPage("en", id);
      const es = helpPage("es", id);
      expect(en.title).not.toBe("");
      expect(es.title).not.toBe("");
      // The two languages say the same: same subtitles, paragraphs and lists.
      expect(blocks(es.body).map((x) => x.kind)).toEqual(blocks(en.body).map((x) => x.kind));
    }
  });

  it("reads subtitles, lists, bold and keys", () => {
    const b = blocks("Intro\n\n## Sub\n- one `Ctrl+C`\n- **two**\n\nEnd");
    expect(b.map((x) => x.kind)).toEqual(["p", "h2", "ul", "p"]);
    expect(pieces("Press `Ctrl+C` or **Copy**.")).toEqual([
      { kind: "text", text: "Press " },
      { kind: "keys", text: "Ctrl+C" },
      { kind: "text", text: " or " },
      { kind: "bold", text: "Copy" },
      { kind: "text", text: "." },
    ]);
  });
});
