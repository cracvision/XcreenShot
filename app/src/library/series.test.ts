import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import { NO_FILTERS } from "./query";
import { openOrEnter, useLibrary, type LibraryItem } from "./state";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const item = (over: Partial<LibraryItem>): LibraryItem => ({
  id: "a",
  name: "a",
  path: "a.xshot",
  kind: "image",
  format: "xshot",
  size: 1,
  modified: "",
  captured: "",
  width: 1,
  height: 1,
  app: null,
  url: null,
  favorite: false,
  tags: [],
  ...over,
});

beforeEach(() => {
  vi.mocked(run).mockReset();
  vi.mocked(run).mockResolvedValue({ items: [], total: 0, count: 0, facets: {}, root: "" });
  useLibrary.setState({ filters: NO_FILTERS, seriesName: "" });
});

describe("interval series in the library (CAP-049)", () => {
  const series = { id: "20261115T100000-s1", index: 3, name: "Panel" };

  it("opening a grouped series shows its captures instead", async () => {
    const open = vi.fn();
    openOrEnter([item({ series, seriesCount: 3 })], open);
    expect(open).not.toHaveBeenCalled();
    expect(useLibrary.getState().filters.series).toBe(series.id);
    expect(useLibrary.getState().seriesName).toBe("Panel");
    await vi.waitFor(() => expect(run).toHaveBeenCalled());
    expect(vi.mocked(run).mock.calls[0]?.[1]).toMatchObject({
      filters: { series: series.id },
      groupSeries: true,
    });
  });

  it("inside the series, or for a lone capture, items open", () => {
    const open = vi.fn();
    openOrEnter([item({ series, seriesCount: 1 })], open);
    expect(open).toHaveBeenCalledTimes(1);
    useLibrary.setState({ filters: { ...NO_FILTERS, series: series.id } });
    openOrEnter([item({ series, seriesCount: 3 })], open);
    expect(open).toHaveBeenCalledTimes(2);
  });
});
