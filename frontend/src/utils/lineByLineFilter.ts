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
