import React, { useState } from "react";
import {
  BookOpen,
  Code2,
  Keyboard,
  Play,
  RefreshCw,
  Sparkles,
  Upload
} from "lucide-react";
import type {
  CompilerAvailability,
  CompilerStandard,
  ConversionMode,
  EditorTheme,
  ExecutionLanguage,
  ExecutionMode,
  ExecutionProvider,
  RustOutputStyle,
  SourceLanguage
} from "../types/studio";
import { Select, SelectItem } from "./ui/Select";
import { SaveMenu } from "./ui/SaveMenu";

interface ToolbarProps {
  sourceLanguage: SourceLanguage;
  onSourceLanguageChange: (language: SourceLanguage) => void;
  conversionMode: ConversionMode;
  onConversionModeChange: (mode: ConversionMode) => void;
  outputStyle: RustOutputStyle;
  onOutputStyleChange: (style: RustOutputStyle) => void;
  onConvertNow: () => void;
  executionProvider: ExecutionProvider;
  onExecutionProviderChange: (provider: ExecutionProvider) => void;
  compilerStandard: CompilerStandard;
  onCompilerStandardChange: (standard: CompilerStandard) => void;
  compilerAvailability: CompilerAvailability | null;
  theme: EditorTheme;
  onThemeChange: (theme: EditorTheme) => void;
  isRustManuallyEdited: boolean;
  onForceResyncRust: () => void;
  onFormatRust: () => void;
  onSaveWorkspace: () => void;
  onOpenExamples: () => void;
  onOpenShortcuts: () => void;
  onImportFile: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onExportSource: () => void;
  onExportRust: () => void;
  onExecute: (language: ExecutionLanguage, mode: ExecutionMode) => void;
  isExecuting: boolean;
  saveStatusMessage: string | null;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  sourceLanguage,
  onSourceLanguageChange,
  conversionMode,
  onConversionModeChange,
  outputStyle,
  onOutputStyleChange,
  onConvertNow,
  executionProvider,
  onExecutionProviderChange,
  compilerStandard,
  onCompilerStandardChange,
  compilerAvailability,
  theme,
  onThemeChange,
  isRustManuallyEdited,
  onForceResyncRust,
  onFormatRust,
  onSaveWorkspace,
  onOpenExamples,
  onOpenShortcuts,
  onImportFile,
  onExportSource,
  onExportRust,
  onExecute,
  isExecuting,
  saveStatusMessage
}) => {
  const sourceLabel = sourceLanguage === "c" ? "C" : "C++";
  const [executionMode, setExecutionMode] = useState<ExecutionMode>("run");
  const standardOptions = sourceLanguage === "c"
    ? [["c11", "C11"], ["c17", "C17"], ["gnu11", "GNU C11"]]
    : [["cpp14", "C++14"], ["cpp17", "C++17"], ["cpp20", "C++20"], ["cpp23", "C++23"]];

  return (
    <header className="studio-toolbar">
      <div className="toolbar-group toolbar-brand">
        <Code2 size={20} className="brand-icon" />
        <span className="brand-title">C/C++ to Rust Playground</span>
        <div className="language-toggle" role="group" aria-label="Source language">
          <button
            type="button"
            className={`toggle-button ${sourceLanguage === "c" ? "active" : ""}`}
            onClick={() => onSourceLanguageChange("c")}
            data-testid="select-c-button"
          >
            C
          </button>
          <button
            type="button"
            className={`toggle-button ${sourceLanguage === "cpp" ? "active" : ""}`}
            onClick={() => onSourceLanguageChange("cpp")}
            data-testid="select-cpp-button"
          >
            C++
          </button>
        </div>

        <select
          className="theme-select"
          value={conversionMode}
          onChange={(event) => onConversionModeChange(event.target.value as ConversionMode)}
          aria-label="Conversion mode"
          title="Live Mode: line-by-line real-time conversion | Manual Mode: convert on button click"
          data-testid="conversion-mode-select"
        >
          <option value="live">Live Mode</option>
          <option value="manual">Manual Mode</option>
        </select>

        <select
          className="theme-select"
          value={outputStyle}
          onChange={(event) => onOutputStyleChange(event.target.value as RustOutputStyle)}
          aria-label="Rust output mode"
          title="Idiomatic Rust (Learning Mode): optional cleaned & rustc-checked Rust | Faithful Rust (Raw Output): uncleaned translator output"
          data-testid="output-style-select"
        >
          <option value="idiomatic">Idiomatic Rust (Learning Mode)</option>
          <option value="faithful">Faithful Rust (Raw Output)</option>
        </select>

        <button
          type="button"
          className="action-button convert-now-button"
          onClick={onConvertNow}
          title="Convert C/C++ to Rust now"
          data-testid="convert-now-button"
        >
          <RefreshCw size={14} />
          <span>Convert</span>
        </button>
      </div>

      <div className="toolbar-group toolbar-actions">
        <button
          type="button"
          className="action-button"
          onClick={onOpenExamples}
          title="Load pre-made C/C++ examples"
          data-testid="examples-button"
        >
          <BookOpen size={15} />
          <span>Examples</span>
        </button>

        <button
          type="button"
          className="action-button"
          onClick={onFormatRust}
          title="Format Rust Code"
          data-testid="format-rust-button"
        >
          <Sparkles size={15} />
          <span>Format</span>
        </button>

        {isRustManuallyEdited && (
          <button
            type="button"
            className="action-button warning-button"
            onClick={onForceResyncRust}
            title="Re-sync Rust from C/C++ (overwrites manual edits)"
            data-testid="resync-rust-button"
          >
            <RefreshCw size={15} />
            <span>Re-sync</span>
          </button>
        )}

        <label className="action-button file-upload-label" title="Import .c, .cpp, or .rs file">
          <Upload size={15} />
          <span>Import</span>
          <input
            type="file"
            accept=".c,.cpp,.cc,.h,.hpp,.rs,.txt"
            onChange={onImportFile}
            style={{ display: "none" }}
          />
        </label>

        <SaveMenu sourceLabel={sourceLanguage} onSaveWorkspace={onSaveWorkspace} onExportSource={onExportSource} onExportRust={onExportRust} statusMessage={saveStatusMessage} />

        <select
          className="theme-select"
          value={theme}
          onChange={(event) => onThemeChange(event.target.value as EditorTheme)}
          aria-label="Theme selector"
          data-testid="theme-select"
        >
          <option value="dark">Dark</option>
          <option value="midnight">Midnight</option>
          <option value="light">Light</option>
        </select>

        <button
          type="button"
          className="icon-button"
          onClick={onOpenShortcuts}
          title="Keyboard Shortcuts"
          aria-label="Keyboard Shortcuts"
        >
          <Keyboard size={16} />
        </button>
      </div>

      <div className="toolbar-group toolbar-execution">
        <select
          className="theme-select"
          value={executionProvider}
          onChange={(event) =>
            onExecutionProviderChange(event.target.value as ExecutionProvider)
          }
          aria-label="Compiler provider"
          title={
            compilerAvailability
              ? `Local compilers — C: ${compilerAvailability.cCompiler ?? "unavailable"}, C++: ${compilerAvailability.cppCompiler ?? "unavailable"}, Rust: ${compilerAvailability.rustCompiler ?? "unavailable"}`
              : "Choose Local (with Online fallback) or Online JDoodle compiler"
          }
          data-testid="provider-select"
        >
          <option value="local">
            {compilerAvailability &&
            (compilerAvailability.cppAvailable || compilerAvailability.rustAvailable)
              ? `Local (${[
                  compilerAvailability.cCompiler,
                  compilerAvailability.cppCompiler,
                  compilerAvailability.rustCompiler
                ]
                  .filter(Boolean)
                  .join(", ")})`
              : "Local (Auto-Fallback)"}
          </option>
          <option value="online">Online (JDoodle)</option>
        </select>

        <Select
          value={executionMode}
          onValueChange={(value) => setExecutionMode(value as ExecutionMode)}
          ariaLabel="Execution action"
          testId="execution-mode-select"
        >
          <SelectItem value="run">Run</SelectItem>
          <SelectItem value="compile">Compile</SelectItem>
        </Select>

        {sourceLanguage !== "c" && sourceLanguage !== "cpp" ? null : (
          <Select
            value={compilerStandard}
            onValueChange={(value) => onCompilerStandardChange(value as CompilerStandard)}
            ariaLabel={`${sourceLabel} language standard`}
            testId="compiler-standard-select"
          >
            {standardOptions.map(([value, label]) => (
              <SelectItem key={value} value={value}>{label}</SelectItem>
            ))}
          </Select>
        )}

        <button
          type="button"
          className="exec-button run-source"
          disabled={isExecuting}
          onClick={() => onExecute(sourceLanguage, executionMode)}
          data-testid="run-source-button"
        >
          <Play size={14} />
          <span>{executionMode === "run" ? "Run" : "Compile"} {sourceLabel}</span>
        </button>
        <button
          type="button"
          className="exec-button run-rust"
          disabled={isExecuting}
          onClick={() => onExecute("rust", executionMode)}
          data-testid="run-rust-button"
        >
          <Play size={14} />
          <span>{executionMode === "run" ? "Run" : "Compile"} Rust</span>
        </button>
      </div>
    </header>
  );
};
