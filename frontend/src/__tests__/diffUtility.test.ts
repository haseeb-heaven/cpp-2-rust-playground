import { describe, expect, it } from "vitest";
import { computeLineDiff } from "../utils/diffUtility";

describe("computeLineDiff", () => {
  it("detects unchanged, added, and removed lines", () => {
    const diff = computeLineDiff("line1\nline2\nline3", "line1\nline2_edited\nline3\nline4");
    expect(diff.some((item) => item.kind === "removed" && item.text === "line2")).toBe(true);
    expect(diff.some((item) => item.kind === "added" && item.text === "line2_edited")).toBe(true);
    expect(diff.some((item) => item.kind === "added" && item.text === "line4")).toBe(true);
    expect(diff.some((item) => item.kind === "unchanged" && item.text === "line1")).toBe(true);
  });
});
