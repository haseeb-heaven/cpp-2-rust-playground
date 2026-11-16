import { describe, expect, it } from "vitest";
import { formatRustCode } from "../utils/rustFormatter";

describe("formatRustCode", () => {
  it("normalizes indentation and trims extra blank lines", () => {
    const messyRust = "fn main() {\nlet x = 5;\n\n\nif x > 0 {\nprintln!(\"{}\", x);\n}\n}";
    const formatted = formatRustCode(messyRust);
    expect(formatted).toBe(
      "fn main() {\n    let x = 5;\n\n    if x > 0 {\n        println!(\"{}\", x);\n    }\n}"
    );
  });
});
