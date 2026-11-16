import React, { useCallback, useEffect, useRef, useState } from "react";
import { Toolbar } from "./components/Toolbar";
import { EditorPane } from "./components/EditorPane";
import { DiffViewer } from "./components/DiffViewer";
import { OutputConsole } from "./components/OutputConsole";
import { ExamplesModal } from "./components/ExamplesModal";
import { ShortcutsModal } from "./components/ShortcutsModal";
import {
  convertSourceCode,
  executeProgramCode,
  fetchCompilerAvailability,
  fetchPremadeExamples
} from "./services/apiClient";
import { mergeIncrementalRustEdits } from "./utils/incrementalMerge";
import { buildLineByLineConversionInput } from "./utils/lineByLineFilter";
import { formatRustCode } from "./utils/rustFormatter";
import {
  downloadTextFile,
  loadWorkspaceFromLocalStorage,
  saveWorkspaceToLocalStorage
} from "./utils/storageUtility";
import type {
  CodeExample,
  CompilerAvailability,
  ConversionDiagnostic,
  ConversionExplanation,
  ConversionMode,
  EditorTheme,
  ExecutionLanguage,
  ExecutionMode,
  ExecutionProvider,
  ExecutionResponse,
  LineMapping,
  RustOutputStyle,
  SourceLanguage
} from "./types/studio";

const DEFAULT_CPP_SOURCE = `#include <iostream>
using namespace std;

int main() {
    string studioName = "RustC-Playground";
    int count = 3;
    for (int index = 0; index < count; index++) {
        cout << studioName << " step " << index << endl;
    }
    return 0;
}`;

const DEFAULT_RUST_CODE = `fn main() {
    let mut studioName: String = String::from("RustC-Playground");
    let mut count: i32 = 3;
    for index in 0..count {
        println!("{} step {}", studioName, index);
    }
    // return 0;
}`;

export const App: React.FC = () => {
  const savedWorkspace = loadWorkspaceFromLocalStorage();

  const [sourceCode, setSourceCode] = useState<string>(
    savedWorkspace?.sourceCode ?? DEFAULT_CPP_SOURCE
  );
  const [sourceLanguage, setSourceLanguage] = useState<SourceLanguage>(
    savedWorkspace?.sourceLanguage ?? "cpp"
  );
  const [rustCode, setRustCode] = useState<string>(
    savedWorkspace?.rustCode ?? DEFAULT_RUST_CODE
  );
  const [lastGeneratedRustCode, setLastGeneratedRustCode] = useState<string>(
    savedWorkspace?.lastGeneratedRustCode ?? DEFAULT_RUST_CODE
  );
  const [isRustManuallyEdited, setIsRustManuallyEdited] = useState<boolean>(
    savedWorkspace?.isRustManuallyEdited ?? false
  );
  const [stdinInput, setStdinInput] = useState<string>(
    savedWorkspace?.stdinInput ?? ""
  );
  const [theme, setTheme] = useState<EditorTheme>(
    savedWorkspace?.theme ?? "dark"
  );
  const [isSwappedPanels, setIsSwappedPanels] = useState<boolean>(
    savedWorkspace?.isSwappedPanels ?? false
  );
  const [splitRatioPercent, setSplitRatioPercent] = useState<number>(
    savedWorkspace?.splitRatioPercent ?? 50
  );

  const [isDiffViewOpen, setIsDiffViewOpen] = useState<boolean>(false);
  const [isExamplesModalOpen, setIsExamplesModalOpen] = useState<boolean>(false);
  const [isShortcutsModalOpen, setIsShortcutsModalOpen] = useState<boolean>(false);
  const [premadeExamples, setPremadeExamples] = useState<CodeExample[]>([]);

  const [lineMappings, setLineMappings] = useState<LineMapping[]>([]);
  const [conversionDiagnostics, setConversionDiagnostics] = useState<ConversionDiagnostic[]>([]);
  const [conversionExplanations, setConversionExplanations] = useState<ConversionExplanation[]>([]);
  const [transpilerEngine, setTranspilerEngine] = useState<string>(
    sourceLanguage === "c"
      ? "c2rust (immunant/c2rust C99/C11 engine)"
      : "cpp2rust (Cpp2Rust/cpp2rust Safe C++ engine)"
  );

  useEffect(() => {
    setTranspilerEngine(
      sourceLanguage === "c"
        ? "c2rust (immunant/c2rust C99/C11 engine)"
        : "cpp2rust (Cpp2Rust/cpp2rust Safe C++ engine)"
    );
  }, [sourceLanguage]);
  const [highlightedSourceLines, setHighlightedSourceLines] = useState<number[]>([]);
  const [highlightedRustLines, setHighlightedRustLines] = useState<number[]>([]);

  const [isExecuting, setIsExecuting] = useState<boolean>(false);
  const [conversionMode, setConversionMode] = useState<ConversionMode>("live");
  const [outputStyle, setOutputStyle] = useState<RustOutputStyle>("idiomatic");
  const [faithfulRustCode, setFaithfulRustCode] = useState<string>(DEFAULT_RUST_CODE);
  const [idiomaticRustCode, setIdiomaticRustCode] = useState<string>(DEFAULT_RUST_CODE);
  const [activeSourceLineNumber, setActiveSourceLineNumber] = useState<number>(1);
  const [executionProvider, setExecutionProvider] = useState<ExecutionProvider>(
    savedWorkspace?.executionProvider ?? "local"
  );
  const [compilerAvailability, setCompilerAvailability] = useState<CompilerAvailability | null>(
    null
  );
  const [executionResult, setExecutionResult] = useState<ExecutionResponse | null>(null);
  const [lastExecutedLabel, setLastExecutedLabel] = useState<string | null>(null);
  const [saveStatusMessage, setSaveStatusMessage] = useState<string | null>(null);

  const splitContainerRef = useRef<HTMLDivElement | null>(null);
  const isDraggingSplitRef = useRef<boolean>(false);
  const debounceTimerRef = useRef<number | null>(null);
  const conversionAbortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    fetchPremadeExamples()
      .then((examples) => setPremadeExamples(examples))
      .catch(() => {
        // Fallback silently if backend is not yet reachable
      });
    fetchCompilerAvailability()
      .then((availability) => setCompilerAvailability(availability))
      .catch(() => {
        // Fallback silently if backend is not yet reachable
      });
  }, []);

  const runConversion = useCallback(
    (
      codeToConvert: string,
      languageToConvert: SourceLanguage,
      styleOverride?: RustOutputStyle
    ) => {
      if (conversionAbortRef.current) {
        conversionAbortRef.current.abort();
      }
      const abortController = new AbortController();
      conversionAbortRef.current = abortController;
      const effectiveStyle = styleOverride ?? outputStyle;

      convertSourceCode(
        codeToConvert,
        languageToConvert,
        abortController.signal,
        effectiveStyle
      )
        .then((conversionResult) => {
          setLineMappings(conversionResult.lineMappings);
          setConversionDiagnostics(conversionResult.diagnostics);
          setConversionExplanations(conversionResult.explanations ?? []);
          if (conversionResult.transpilerEngine) {
            setTranspilerEngine(conversionResult.transpilerEngine);
          }
          if (conversionResult.faithfulRustCode !== undefined) {
            setFaithfulRustCode(conversionResult.faithfulRustCode);
          }
          if (conversionResult.idiomaticRustCode !== undefined) {
            setIdiomaticRustCode(conversionResult.idiomaticRustCode);
          }

          const targetGenerated =
            effectiveStyle === "faithful"
              ? (conversionResult.faithfulRustCode ?? conversionResult.generatedRustCode)
              : (conversionResult.idiomaticRustCode ?? conversionResult.generatedRustCode);

          const mergeResult = mergeIncrementalRustEdits(
            lastGeneratedRustCode,
            targetGenerated,
            rustCode,
            isRustManuallyEdited
          );
          setLastGeneratedRustCode(targetGenerated);
          setRustCode(mergeResult.mergedRustCode);
        })
        .catch((error: unknown) => {
          if (error instanceof DOMException && error.name === "AbortError") {
            return;
          }
        });
    },
    [lastGeneratedRustCode, rustCode, isRustManuallyEdited, outputStyle]
  );

  const handleOutputStyleChange = useCallback(
    (nextStyle: RustOutputStyle) => {
      setOutputStyle(nextStyle);
      const cachedTarget =
        nextStyle === "faithful" ? faithfulRustCode : idiomaticRustCode;
      if (!isRustManuallyEdited && cachedTarget) {
        setLastGeneratedRustCode(cachedTarget);
        setRustCode(cachedTarget);
      }
      if (conversionMode === "live") {
        const inputForMode = buildLineByLineConversionInput(
          sourceCode,
          activeSourceLineNumber
        );
        runConversion(inputForMode, sourceLanguage, nextStyle);
      }
    },
    [
      faithfulRustCode,
      idiomaticRustCode,
      isRustManuallyEdited,
      conversionMode,
      sourceCode,
      activeSourceLineNumber,
      sourceLanguage,
      runConversion
    ]
  );

  useEffect(() => {
    if (conversionMode === "manual") {
      if (debounceTimerRef.current !== null) {
        window.clearTimeout(debounceTimerRef.current);
        debounceTimerRef.current = null;
      }
      if (conversionAbortRef.current) {
        conversionAbortRef.current.abort();
        conversionAbortRef.current = null;
      }
      return;
    }

    if (debounceTimerRef.current !== null) {
      window.clearTimeout(debounceTimerRef.current);
    }

    const inputForMode = buildLineByLineConversionInput(
      sourceCode,
      activeSourceLineNumber
    );

    debounceTimerRef.current = window.setTimeout(() => {
      runConversion(inputForMode, sourceLanguage);
    }, 300);

    return () => {
      if (debounceTimerRef.current !== null) {
        window.clearTimeout(debounceTimerRef.current);
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sourceCode, sourceLanguage, conversionMode, activeSourceLineNumber]);

  const handleConvertNow = useCallback(() => {
    if (debounceTimerRef.current !== null) {
      window.clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }
    runConversion(sourceCode, sourceLanguage);
  }, [runConversion, sourceCode, sourceLanguage]);

  const handleRustEditorChange = useCallback(
    (nextValue: string) => {
      setRustCode(nextValue);
      setIsRustManuallyEdited(nextValue !== lastGeneratedRustCode);
    },
    [lastGeneratedRustCode]
  );

  const handleForceResyncRust = useCallback(() => {
    setRustCode(lastGeneratedRustCode);
    setIsRustManuallyEdited(false);
  }, [lastGeneratedRustCode]);

  const handleFormatRust = useCallback(() => {
    const formatted = formatRustCode(rustCode);
    setRustCode(formatted);
    setIsRustManuallyEdited(formatted !== lastGeneratedRustCode);
  }, [rustCode, lastGeneratedRustCode]);

  const handleSaveWorkspace = useCallback(() => {
    saveWorkspaceToLocalStorage({
      sourceCode,
      sourceLanguage,
      rustCode,
      lastGeneratedRustCode,
      isRustManuallyEdited,
      stdinInput,
      theme,
      isSwappedPanels,
      splitRatioPercent,
      executionProvider,
      savedAtTimestamp: new Date().toISOString()
    });
    setSaveStatusMessage("Saved!");
    window.setTimeout(() => setSaveStatusMessage(null), 1500);
  }, [
    sourceCode,
    sourceLanguage,
    rustCode,
    lastGeneratedRustCode,
    isRustManuallyEdited,
    stdinInput,
    theme,
    isSwappedPanels,
    splitRatioPercent,
    executionProvider
  ]);

  const handleExecute = useCallback(
    async (language: ExecutionLanguage, mode: ExecutionMode) => {
      const codeToRun = language === "rust" ? rustCode : sourceCode;
      const languageLabel =
        language === "c" ? "C" : language === "cpp" ? "C++" : "Rust";
      const modeLabel = mode === "compile" ? "Compile" : "Run";

      setIsExecuting(true);
      setLastExecutedLabel(`${modeLabel} ${languageLabel}`);

      try {
        const result = await executeProgramCode(
          codeToRun,
          language,
          mode,
          stdinInput,
          executionProvider
        );
        setExecutionResult(result);
      } catch (error: unknown) {
        const errorMessage =
          error instanceof Error ? error.message : "Network error during execution.";
        setExecutionResult({
          success: false,
          output: "",
          errorOutput: errorMessage,
          statusCode: 500,
          memoryUsed: null,
          cpuTimeSeconds: null,
          executionTimeMilliseconds: 0,
          diagnostics: [
            {
              lineNumber: 1,
              severity: "error",
              category: "network",
              message: errorMessage,
              suggestedFix: "Ensure the Rust backend server is running on port 8080."
            }
          ],
          timedOut: false,
          usedProvider: executionProvider
        });
      } finally {
        setIsExecuting(false);
      }
    },
    [rustCode, sourceCode, stdinInput, executionProvider]
  );

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const hasModifier = event.ctrlKey || event.metaKey;
      if (!hasModifier) {
        return;
      }

      if (event.key === "Enter" && event.shiftKey) {
        event.preventDefault();
        void handleExecute(sourceLanguage, "run");
      } else if (event.key === "Enter") {
        event.preventDefault();
        void handleExecute("rust", "run");
      } else if (event.key.toLowerCase() === "b") {
        event.preventDefault();
        void handleExecute("rust", "compile");
      } else if (event.key.toLowerCase() === "s") {
        event.preventDefault();
        handleSaveWorkspace();
      } else if (event.key === "\\") {
        event.preventDefault();
        setIsSwappedPanels((previous) => !previous);
      } else if (event.key.toLowerCase() === "d") {
        event.preventDefault();
        setIsDiffViewOpen((previous) => !previous);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleExecute, handleSaveWorkspace, sourceLanguage]);

  const handleSourceCursorLine = useCallback(
    (lineNumber: number) => {
      setActiveSourceLineNumber(lineNumber);
      const mapping = lineMappings.find((item) => item.sourceLine === lineNumber);
      setHighlightedSourceLines([lineNumber]);
      if (mapping) {
        const mappedRustLines: number[] = [];
        for (let line = mapping.rustStartLine; line <= mapping.rustEndLine; line += 1) {
          mappedRustLines.push(line);
        }
        setHighlightedRustLines(mappedRustLines);
      } else {
        setHighlightedRustLines([]);
      }
    },
    [lineMappings]
  );

  const handleRustCursorLine = useCallback(
    (lineNumber: number) => {
      const mapping = lineMappings.find(
        (item) => lineNumber >= item.rustStartLine && lineNumber <= item.rustEndLine
      );
      setHighlightedRustLines([lineNumber]);
      if (mapping) {
        setHighlightedSourceLines([mapping.sourceLine]);
      } else {
        setHighlightedSourceLines([]);
      }
    },
    [lineMappings]
  );

  const handleSelectLineMapping = useCallback(
    (sourceLine: number, rustStartLine: number, rustEndLine: number) => {
      setHighlightedSourceLines([sourceLine]);
      const rustLines: number[] = [];
      for (let line = rustStartLine; line <= rustEndLine; line += 1) {
        rustLines.push(line);
      }
      setHighlightedRustLines(rustLines);
    },
    []
  );

  const handleImportFile = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = event.target.files?.[0];
    if (!selectedFile) {
      return;
    }

    const reader = new FileReader();
    reader.onload = () => {
      const content = typeof reader.result === "string" ? reader.result : "";
      if (selectedFile.name.endsWith(".rs")) {
        setRustCode(content);
        setIsRustManuallyEdited(true);
      } else if (selectedFile.name.endsWith(".c")) {
        setSourceLanguage("c");
        setSourceCode(content);
        setIsRustManuallyEdited(false);
      } else {
        setSourceLanguage("cpp");
        setSourceCode(content);
        setIsRustManuallyEdited(false);
      }
    };
    reader.readAsText(selectedFile);
    event.target.value = "";
  }, []);

  const handleSelectExample = useCallback((example: CodeExample) => {
    setSourceLanguage(example.sourceLanguage);
    setSourceCode(example.sourceCode);
    setStdinInput(example.defaultStdin);
    setIsRustManuallyEdited(false);
    setExecutionResult(null);
  }, []);

  const startSplitDrag = () => {
    isDraggingSplitRef.current = true;
  };

  useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      if (!isDraggingSplitRef.current || !splitContainerRef.current) {
        return;
      }
      const rect = splitContainerRef.current.getBoundingClientRect();
      const relativeX = event.clientX - rect.left;
      const percentage = Math.min(75, Math.max(25, (relativeX / rect.width) * 100));
      setSplitRatioPercent(percentage);
    };

    const handleMouseUp = () => {
      isDraggingSplitRef.current = false;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, []);

  const sourceErrorLines = conversionDiagnostics
    .filter((diag) => diag.severity === "error")
    .map((diag) => diag.lineNumber);

  const sourceEditorPane = (
    <EditorPane
      title={sourceLanguage === "c" ? "C Source" : "C++ Source"}
      language="cpp"
      value={sourceCode}
      onChange={setSourceCode}
      theme={theme}
      highlightedLines={highlightedSourceLines}
      errorLines={sourceErrorLines}
      onCursorLineChange={handleSourceCursorLine}
      badgeText={sourceLanguage.toUpperCase()}
      testId="source-editor-pane"
    />
  );

  const rustEditorPane = (
    <EditorPane
      title="Generated Rust"
      language="rust"
      value={rustCode}
      onChange={handleRustEditorChange}
      theme={theme}
      highlightedLines={highlightedRustLines}
      onCursorLineChange={handleRustCursorLine}
      secondaryBadgeText={
        sourceLanguage === "c" ? "Engine: c2rust" : "Engine: cpp2rust"
      }
      secondaryBadgeTitle={transpilerEngine}
      secondaryBadgeTestId="transpiler-engine-badge"
      badgeText={isRustManuallyEdited ? "Manual Edits Preserved" : "Auto-Synced"}
      testId="rust-editor-pane"
    />
  );

  const leftPane = isSwappedPanels ? rustEditorPane : sourceEditorPane;
  const rightPane = isSwappedPanels ? sourceEditorPane : rustEditorPane;

  return (
    <div className={`studio-shell theme-${theme}`} data-testid="studio-shell">
      <Toolbar
        sourceLanguage={sourceLanguage}
        onSourceLanguageChange={setSourceLanguage}
        conversionMode={conversionMode}
        onConversionModeChange={setConversionMode}
        outputStyle={outputStyle}
        onOutputStyleChange={handleOutputStyleChange}
        onConvertNow={handleConvertNow}
        executionProvider={executionProvider}
        onExecutionProviderChange={setExecutionProvider}
        compilerAvailability={compilerAvailability}
        theme={theme}
        onThemeChange={setTheme}
        isDiffViewOpen={isDiffViewOpen}
        onToggleDiffView={() => setIsDiffViewOpen((previous) => !previous)}
        isSwappedPanels={isSwappedPanels}
        onSwapPanels={() => setIsSwappedPanels((previous) => !previous)}
        isRustManuallyEdited={isRustManuallyEdited}
        onForceResyncRust={handleForceResyncRust}
        onFormatRust={handleFormatRust}
        onSaveWorkspace={handleSaveWorkspace}
        onOpenExamples={() => setIsExamplesModalOpen(true)}
        onOpenShortcuts={() => setIsShortcutsModalOpen(true)}
        onImportFile={handleImportFile}
        onExportSource={() =>
          downloadTextFile(`main.${sourceLanguage}`, sourceCode)
        }
        onExportRust={() => downloadTextFile("main.rs", rustCode)}
        onExecute={handleExecute}
        isExecuting={isExecuting}
        saveStatusMessage={saveStatusMessage}
      />

      <main className="studio-main">
        {isDiffViewOpen ? (
          <DiffViewer
            leftTitle="Auto-Generated Rust"
            rightTitle="Current Rust Editor"
            leftText={lastGeneratedRustCode}
            rightText={rustCode}
          />
        ) : (
          <div className="split-workspace" ref={splitContainerRef}>
            <div className="split-pane" style={{ width: `${splitRatioPercent}%` }}>
              {leftPane}
            </div>
            <div
              className="split-divider"
              onMouseDown={startSplitDrag}
              role="separator"
              aria-orientation="vertical"
              title="Drag to resize panels"
              data-testid="split-divider"
            />
            <div
              className="split-pane"
              style={{ width: `${100 - splitRatioPercent}%` }}
            >
              {rightPane}
            </div>
          </div>
        )}

        <OutputConsole
          stdinInput={stdinInput}
          onStdinChange={setStdinInput}
          executionProvider={executionProvider}
          executionResult={executionResult}
          lastExecutedLabel={lastExecutedLabel}
          isExecuting={isExecuting}
          conversionDiagnostics={conversionDiagnostics}
          conversionExplanations={conversionExplanations}
          lineMappings={lineMappings}
          onSelectLineMapping={handleSelectLineMapping}
        />
      </main>

      <ExamplesModal
        isOpen={isExamplesModalOpen}
        examples={premadeExamples}
        onSelectExample={handleSelectExample}
        onClose={() => setIsExamplesModalOpen(false)}
      />

      <ShortcutsModal
        isOpen={isShortcutsModalOpen}
        onClose={() => setIsShortcutsModalOpen(false)}
      />
    </div>
  );
};

export default App;
