export type DiffLineKind = "unchanged" | "added" | "removed";

export interface DiffLineItem {
  kind: DiffLineKind;
  leftLineNumber: number | null;
  rightLineNumber: number | null;
  text: string;
}

export function computeLineDiff(leftText: string, rightText: string): DiffLineItem[] {
  const leftLines = leftText.split("\n");
  const rightLines = rightText.split("\n");
  const leftCount = leftLines.length;
  const rightCount = rightLines.length;

  const lcsMatrix: number[][] = Array.from({ length: leftCount + 1 }, () =>
    Array<number>(rightCount + 1).fill(0)
  );

  for (let leftIndex = leftCount - 1; leftIndex >= 0; leftIndex -= 1) {
    for (let rightIndex = rightCount - 1; rightIndex >= 0; rightIndex -= 1) {
      if (leftLines[leftIndex] === rightLines[rightIndex]) {
        lcsMatrix[leftIndex][rightIndex] = lcsMatrix[leftIndex + 1][rightIndex + 1] + 1;
      } else {
        lcsMatrix[leftIndex][rightIndex] = Math.max(
          lcsMatrix[leftIndex + 1][rightIndex],
          lcsMatrix[leftIndex][rightIndex + 1]
        );
      }
    }
  }

  const diffItems: DiffLineItem[] = [];
  let leftPointer = 0;
  let rightPointer = 0;

  while (leftPointer < leftCount && rightPointer < rightCount) {
    if (leftLines[leftPointer] === rightLines[rightPointer]) {
      diffItems.push({
        kind: "unchanged",
        leftLineNumber: leftPointer + 1,
        rightLineNumber: rightPointer + 1,
        text: leftLines[leftPointer]
      });
      leftPointer += 1;
      rightPointer += 1;
    } else if (lcsMatrix[leftPointer + 1][rightPointer] >= lcsMatrix[leftPointer][rightPointer + 1]) {
      diffItems.push({
        kind: "removed",
        leftLineNumber: leftPointer + 1,
        rightLineNumber: null,
        text: leftLines[leftPointer]
      });
      leftPointer += 1;
    } else {
      diffItems.push({
        kind: "added",
        leftLineNumber: null,
        rightLineNumber: rightPointer + 1,
        text: rightLines[rightPointer]
      });
      rightPointer += 1;
    }
  }

  while (leftPointer < leftCount) {
    diffItems.push({
      kind: "removed",
      leftLineNumber: leftPointer + 1,
      rightLineNumber: null,
      text: leftLines[leftPointer]
    });
    leftPointer += 1;
  }

  while (rightPointer < rightCount) {
    diffItems.push({
      kind: "added",
      leftLineNumber: null,
      rightLineNumber: rightPointer + 1,
      text: rightLines[rightPointer]
    });
    rightPointer += 1;
  }

  return diffItems;
}
