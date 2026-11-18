import React, { useMemo } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { cpp } from "@codemirror/lang-cpp";
import { rust } from "@codemirror/lang-rust";
import { acceptCompletion, autocompletion, type CompletionContext } from "@codemirror/autocomplete";
import { indentMore } from "@codemirror/commands";
import { oneDark } from "@codemirror/theme-one-dark";
import { Decoration, EditorView, keymap } from "@codemirror/view";
import { Prec, StateField } from "@codemirror/state";
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
  if (!word && !context.explicit) {
    return null;
  }
  if (!word || word.from === word.to) {
    return { from: context.pos, options: cppCompletionOptions, filter: false };
  }
  // Filter here so the popup only narrows as the user types; returning every
  // option for every word leaves no active entry for Tab to accept.
  const query = word.text.toLowerCase();
  const options = cppCompletionOptions.filter((option) =>
    option.label.toLowerCase().includes(query)
  );
  if (options.length === 0) {
    return null;
  }
  return { from: word.from, options };
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
  const languageExtension = useMemo(() => (language === "cpp" ? cpp() : rust()), [language]);

  // Highlighting changes with the cursor/diagnostics; completion must not, or
  // every re-render reconfigures the editor and resets an open popup.
  const highlightExtension = useMemo(
    () => createLineHighlightExtension(highlightedLines, errorLines),
    [highlightedLines, errorLines]
  );

  const cursorListener = useMemo(
    () =>
      EditorView.updateListener.of((update) => {
        if (update.selectionSet && onCursorLineChange) {
          const cursorPosition = update.state.selection.main.head;
          const lineNumber = update.state.doc.lineAt(cursorPosition).number;
          onCursorLineChange(lineNumber);
        }
      }),
    [onCursorLineChange]
  );

  const completionExtension = useMemo(
    () =>
      language === "cpp"
        ? [
            autocompletion({ override: [completeCpp] }),
            // Tab accepts an open completion and otherwise keeps indenting. The
            // component-level `indentWithTab` is disabled so this binding is not
            // shadowed by the highest-precedence default indentation binding.
            Prec.high(
              keymap.of([
                { key: "Tab", run: (view) => acceptCompletion(view) || indentMore(view) }
              ])
            )
          ]
        : [],
    [language]
  );

  const extensions = useMemo(
    () => [languageExtension, highlightExtension, cursorListener, ...completionExtension],
    [languageExtension, highlightExtension, cursorListener, completionExtension]
  );

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
          indentWithTab={false}
          basicSetup={{
            lineNumbers: true,
            highlightActiveLineGutter: true,
            foldGutter: true,
            // We install our own `autocompletion` extension for C++ below; a second
            // built-in instance renders a popup that its own keymap cannot accept.
            autocompletion: false,
            bracketMatching: true
          }}
        />
      </div>
    </section>
  );
};
