import React, { useMemo } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { cpp } from "@codemirror/lang-cpp";
import { rust } from "@codemirror/lang-rust";
import { oneDark } from "@codemirror/theme-one-dark";
import { Decoration, EditorView } from "@codemirror/view";
import { StateField } from "@codemirror/state";
import type { EditorTheme } from "../types/studio";

interface EditorPaneProps {
  title: string;
  language: "cpp" | "rust";
  value: string;
  onChange: (nextValue: string) => void;
  theme: EditorTheme;
  highlightedLines: number[];
  onCursorLineChange?: (lineNumber: number) => void;
  badgeText?: string;
  testId: string;
}

function createLineHighlightExtension(highlightedLines: number[]) {
  const lineDecoration = Decoration.line({ class: "cm-mapped-line-highlight" });

  return StateField.define({
    create(state) {
      return buildDecorations(state.doc, highlightedLines, lineDecoration);
    },
    update(_decorations, transaction) {
      return buildDecorations(transaction.state.doc, highlightedLines, lineDecoration);
    },
    provide: (field) => EditorView.decorations.from(field)
  });
}

function buildDecorations(
  document: { lines: number; line: (number: number) => { from: number } },
  highlightedLines: number[],
  decoration: Decoration
) {
  const ranges = [];
  for (const lineNumber of highlightedLines) {
    if (lineNumber >= 1 && lineNumber <= document.lines) {
      const lineInfo = document.line(lineNumber);
      ranges.push(decoration.range(lineInfo.from));
    }
  }
  return Decoration.set(ranges, true);
}

export const EditorPane: React.FC<EditorPaneProps> = ({
  title,
  language,
  value,
  onChange,
  theme,
  highlightedLines,
  onCursorLineChange,
  badgeText,
  testId
}) => {
  const extensions = useMemo(() => {
    const languageExtension = language === "cpp" ? cpp() : rust();
    const highlightExtension = createLineHighlightExtension(highlightedLines);
    const cursorListener = EditorView.updateListener.of((update) => {
      if (update.selectionSet && onCursorLineChange) {
        const cursorPosition = update.state.selection.main.head;
        const lineNumber = update.state.doc.lineAt(cursorPosition).number;
        onCursorLineChange(lineNumber);
      }
    });
    return [languageExtension, highlightExtension, cursorListener];
  }, [language, highlightedLines, onCursorLineChange]);

  return (
    <section className="editor-pane" data-testid={testId}>
      <div className="pane-header">
        <span className="pane-title">{title}</span>
        {badgeText && <span className="pane-badge">{badgeText}</span>}
      </div>
      <div className="pane-editor-container">
        <CodeMirror
          value={value}
          height="100%"
          theme={theme === "light" ? "light" : oneDark}
          extensions={extensions}
          onChange={onChange}
          basicSetup={{
            lineNumbers: true,
            highlightActiveLineGutter: true,
            foldGutter: true,
            autocompletion: true,
            bracketMatching: true
          }}
        />
      </div>
    </section>
  );
};
