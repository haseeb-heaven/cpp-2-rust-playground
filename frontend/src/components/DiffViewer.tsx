import React, { useMemo } from "react";
import { computeLineDiff } from "../utils/diffUtility";

interface DiffViewerProps {
  leftTitle: string;
  rightTitle: string;
  leftText: string;
  rightText: string;
}

export const DiffViewer: React.FC<DiffViewerProps> = ({
  leftTitle,
  rightTitle,
  leftText,
  rightText
}) => {
  const diffLines = useMemo(
    () => computeLineDiff(leftText, rightText),
    [leftText, rightText]
  );

  return (
    <div className="diff-viewer" data-testid="diff-viewer">
      <div className="diff-header">
        <span>
          Diff: <strong>{leftTitle}</strong> vs <strong>{rightTitle}</strong>
        </span>
        <span className="diff-legend">
          <span className="legend-added">+ Added / Changed</span>
          <span className="legend-removed">- Removed</span>
        </span>
      </div>
      <div className="diff-body">
        {diffLines.map((item, index) => (
          <div key={index} className={`diff-row diff-${item.kind}`}>
            <span className="diff-line-num">{item.leftLineNumber ?? ""}</span>
            <span className="diff-line-num">{item.rightLineNumber ?? ""}</span>
            <span className="diff-marker">
              {item.kind === "added" ? "+" : item.kind === "removed" ? "-" : " "}
            </span>
            <pre className="diff-code">{item.text}</pre>
          </div>
        ))}
      </div>
    </div>
  );
};
