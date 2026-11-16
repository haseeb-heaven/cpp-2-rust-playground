import { describe, expect, it } from "vitest";
import { mergeIncrementalRustEdits } from "../utils/incrementalMerge";

describe("mergeIncrementalRustEdits", () => {
  it("replaces code when Rust has not been manually edited", () => {
    const result = mergeIncrementalRustEdits(
      "fn main() {\n    println!(\"old\");\n}",
      "fn main() {\n    println!(\"new\");\n}",
      "fn main() {\n    println!(\"old\");\n}",
      false
    );

    expect(result.mergedRustCode).toBe("fn main() {\n    println!(\"new\");\n}");
    expect(result.preservedManualEdits).toBe(false);
  });

  it("preserves manually edited lines while updating untouched lines", () => {
    const previousGenerated = "fn main() {\n    let mut x: i32 = 1;\n    println!(\"{}\", x);\n}";
    const currentRust = "fn main() {\n    let mut x: i32 = 1;\n    println!(\"Custom: {}\", x);\n}";
    const nextGenerated = "fn main() {\n    let mut x: i32 = 42;\n    println!(\"{}\", x);\n}";

    const result = mergeIncrementalRustEdits(
      previousGenerated,
      nextGenerated,
      currentRust,
      true
    );

    expect(result.preservedManualEdits).toBe(true);
    expect(result.mergedRustCode).toContain("let mut x: i32 = 42;");
    expect(result.mergedRustCode).toContain("println!(\"Custom: {}\", x);");
  });
});
