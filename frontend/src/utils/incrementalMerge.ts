export interface IncrementalMergeResult {
  mergedRustCode: string;
  preservedManualEdits: boolean;
}

export function mergeIncrementalRustEdits(
  previousGeneratedRustCode: string,
  newGeneratedRustCode: string,
  currentRustCode: string,
  isRustManuallyEdited: boolean
): IncrementalMergeResult {
  if (!isRustManuallyEdited || currentRustCode === previousGeneratedRustCode) {
    return {
      mergedRustCode: newGeneratedRustCode,
      preservedManualEdits: false
    };
  }

  const previousLines = previousGeneratedRustCode.split("\n");
  const nextLines = newGeneratedRustCode.split("\n");
  const currentLines = currentRustCode.split("\n");

  const maxLineCount = Math.max(previousLines.length, nextLines.length, currentLines.length);
  const mergedLines: string[] = [];

  for (let index = 0; index < maxLineCount; index += 1) {
    const previousLine = previousLines[index];
    const nextLine = nextLines[index];
    const currentLine = currentLines[index];

    const lineWasManuallyEdited =
      currentLine !== undefined && currentLine !== previousLine;

    if (lineWasManuallyEdited) {
      mergedLines.push(currentLine);
    } else if (nextLine !== undefined) {
      mergedLines.push(nextLine);
    }
  }

  return {
    mergedRustCode: mergedLines.join("\n"),
    preservedManualEdits: true
  };
}
