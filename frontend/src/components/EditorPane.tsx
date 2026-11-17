import React, { useMemo } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { cpp } from "@codemirror/lang-cpp";
import { rust } from "@codemirror/lang-rust";
import { acceptCompletion, autocompletion, type CompletionContext } from "@codemirror/autocomplete";
import { oneDark } from "@codemirror/theme-one-dark";
import { Decoration, EditorView, keymap } from "@codemirror/view";
import { StateField } from "@codemirror/state";
import type { EditorTheme } from "../types/studio";

interface EditorPaneProps {
  title: string;
  language: "cpp" | "rust";
  value: string;
  onChange: (nextValue: string) => void;
  theme: EditorTheme;
  highlightedLines: number[];
  errorLines?: number[];
  onCursorLineChange?: (lineNumber: number) => void;
  badgeText?: string;
  secondaryBadgeText?: string;
  secondaryBadgeTitle?: string;
  secondaryBadgeTestId?: string;
  testId: string;
}

const cppCompletionOptions = [
  { label: "#include <iostream>", type: "keyword", detail: "standard I/O" },
  { label: "#include <vector>", type: "keyword", detail: "dynamic array" },
  { label: "#include <string>", type: "keyword", detail: "strings" },
  { label: "#include <unordered_map>", type: "keyword", detail: "hash map" },
  { label: "#include <map>", type: "keyword", detail: "ordered map" },
  { label: "#include <stdio.h>", type: "keyword", detail: "C I/O" },
  { label: "#include <stdlib.h>", type: "keyword", detail: "C utilities" },
  { label: "std::vector", type: "class", detail: "growable sequence" },
  { label: "std::unordered_map", type: "class", detail: "key-value map" },
  { label: "std::string", type: "class", detail: "owned string" },
  { label: "push_back", type: "method", detail: "append an item", apply: "push_back(${value});" },
  { label: "erase", type: "method", detail: "remove an item", apply: "erase(${position});" },
  { label: "find", type: "method", detail: "look up an item", apply: "find(${key})" },
  { label: "clear", type: "method", detail: "remove all items", apply: "clear();" },
  { label: "printf", type: "function", detail: "C formatted output", apply: "printf(\"${format}\\n\");" },
  { label: "std::cout", type: "variable", detail: "C++ output stream" }
];

function completeCpp(context: CompletionContext) {
  const word = context.matchBefore(/[A-Za-z_:#][A-Za-z0-9_:#<>.]*/);
  if (!context.explicit && !word) {
    return null;
  }
  return {
    from: word?.from ?? context.pos,
    options: cppCompletionOptions
  };
}

function createLineHighlightExtension(highlightedLines: number[], errorLines: number[]) {
  const mappedLineDecoration = Decoration.line({ class: "cm-mapped-line-highlight" });
  const errorLineDecoration = Decoration.line({ class: "cm-error-line-highlight" });

  return StateField.define({
    create(state) {
      return buildDecorations(
        state.doc,
        highlightedLines,
        errorLines,
        mappedLineDecoration,
        errorLineDecoration
      );
    },
    update(_decorations, transaction) {
      return buildDecorations(
        transaction.state.doc,
        highlightedLines,
        errorLines,
        mappedLineDecoration,
        errorLineDecoration
      );
    },
    provide: (field) => EditorView.decorations.from(field)
  });
}

function buildDecorations(
  document: { lines: number; line: (number: number) => { from: number } },
  highlightedLines: number[],
  errorLines: number[],
  mappedDecoration: Decoration,
  errorDecoration: Decoration
) {
  const ranges = [];
  for (const lineNumber of highlightedLines) {
    if (lineNumber >= 1 && lineNumber <= document.lines) {
      const lineInfo = document.line(lineNumber);
      ranges.push(mappedDecoration.range(lineInfo.from));
    }
  }
  for (const lineNumber of errorLines) {
    if (lineNumber >= 1 && lineNumber <= document.lines) {
      const lineInfo = document.line(lineNumber);
      ranges.push(errorDecoration.range(lineInfo.from));
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
  errorLines = [],
  onCursorLineChange,
  badgeText,
  secondaryBadgeText,
  secondaryBadgeTitle,
  secondaryBadgeTestId,
  testId
}) => {
  const extensions = useMemo(() => {
    const languageExtension = language === "cpp" ? cpp() : rust();
    const highlightExtension = createLineHighlightExtension(highlightedLines, errorLines);
    const cursorListener = EditorView.updateListener.of((update) => {
      if (update.selectionSet && onCursorLineChange) {
        const cursorPosition = update.state.selection.main.head;
        const lineNumber = update.state.doc.lineAt(cursorPosition).number;
        onCursorLineChange(lineNumber);
      }
    });
    const completionExtension = language === "cpp"
      ? [
          autocompletion({ override: [completeCpp] }),
          keymap.of([{ key: "Tab", run: acceptCompletion }])
        ]
      : [];
    return [languageExtension, highlightExtension, cursorListener, ...completionExtension];
  }, [language, highlightedLines, errorLines, onCursorLineChange]);

  return (
    <section className="editor-pane" data-testid={testId}>
      <div className="pane-header">
        <span className="pane-title">{title}</span>
        {secondaryBadgeText && (
          <span
            className="pane-badge"
            title={secondaryBadgeTitle}
            data-testid={secondaryBadgeTestId}
          >
            {secondaryBadgeText}
          </span>
        )}
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
