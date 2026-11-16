export function formatRustCode(rustCode: string): string {
  const rawLines = rustCode.split("\n");
  const formattedLines: string[] = [];
  let indentationLevel = 0;
  let consecutiveBlankLines = 0;

  for (const rawLine of rawLines) {
    const trimmed = rawLine.trim();

    if (trimmed === "") {
      consecutiveBlankLines += 1;
      if (consecutiveBlankLines <= 1 && formattedLines.length > 0) {
        formattedLines.push("");
      }
      continue;
    }

    consecutiveBlankLines = 0;

    if (trimmed.startsWith("}")) {
      indentationLevel = Math.max(0, indentationLevel - 1);
    }

    const indentPrefix = "    ".repeat(indentationLevel);
    formattedLines.push(`${indentPrefix}${trimmed}`);

    const openBraces = (trimmed.match(/\{/g) ?? []).length;
    const closeBraces = (trimmed.match(/\}/g) ?? []).length;
    if (trimmed.startsWith("}")) {
      indentationLevel = Math.max(0, indentationLevel + openBraces - Math.max(0, closeBraces - 1));
    } else {
      indentationLevel = Math.max(0, indentationLevel + openBraces - closeBraces);
    }
  }

  while (formattedLines.length > 0 && formattedLines[formattedLines.length - 1] === "") {
    formattedLines.pop();
  }

  return formattedLines.join("\n");
}
