function hasBalancedParens(text: string): boolean {
  let depth = 0;
  let inQuotes = false;
  let escaped = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (inQuotes) {
      if (ch === "\"" && !escaped) {
        inQuotes = false;
      }
      escaped = ch === "\\" && !escaped;
      continue;
    }
    if (ch === "\"") {
      inQuotes = true;
    } else if (ch === "(") {
      depth += 1;
    } else if (ch === ")") {
      depth -= 1;
      if (depth < 0) {
        return false;
      }
    }
  }
  return depth === 0;
}

export function isLineCompletedForConversion(lineText: string): boolean {
  const trimmed = lineText.trim();
  if (trimmed === "" || trimmed.startsWith("//") || trimmed.startsWith("/*")) {
    return true;
  }
  if (trimmed.startsWith("#")) {
    return trimmed.endsWith(">") || trimmed.endsWith("\"") || /\s+/.test(trimmed);
  }
  if (trimmed.startsWith("template") && trimmed.endsWith(">")) {
    return true;
  }
  if (trimmed === "else" || trimmed === "do") {
    return true;
  }
  if (trimmed.endsWith(")") && hasBalancedParens(trimmed)) {
    return true;
  }
  return (
    trimmed.endsWith(";") ||
    trimmed.endsWith("{") ||
    trimmed.endsWith("}") ||
    trimmed.endsWith("};") ||
    trimmed.endsWith(":")
  );
}

export function buildLineByLineConversionInput(
  sourceCode: string,
  activeCursorLineNumber: number
): string {
  const lines = sourceCode.split("\n");
  return lines
    .map((lineText, zeroBasedIndex) => {
      const lineNumber = zeroBasedIndex + 1;
      if (lineNumber === activeCursorLineNumber && !isLineCompletedForConversion(lineText)) {
        return "";
      }
      return lineText;
    })
    .join("\n");
}
