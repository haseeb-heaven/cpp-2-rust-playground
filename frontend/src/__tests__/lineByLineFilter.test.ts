import { describe, expect, it } from "vitest";
import {
  buildLineByLineConversionInput,
  isLineCompletedForConversion
} from "../utils/lineByLineFilter";

describe("lineByLineFilter", () => {
  it("detects completed vs incomplete lines", () => {
    expect(isLineCompletedForConversion("int x = 5;")).toBe(true);
    expect(isLineCompletedForConversion("for (int i = 0; i < 5; i++) {")).toBe(true);
    expect(isLineCompletedForConversion("}")).toBe(true);
    expect(isLineCompletedForConversion("std::vector<float> fvec")).toBe(false);
    expect(isLineCompletedForConversion("for (auto& item")).toBe(false);
  });

  it("excludes an incomplete active cursor line while keeping completed lines", () => {
    const code = "int main() {\n    int a = 1;\n    std::vector<float> fvec\n}";
    const filtered = buildLineByLineConversionInput(code, 3);
    expect(filtered).toBe("int main() {\n    int a = 1;\n\n}");
  });
});
