import React from "react";
import {
  ArrowLeftRight,
  BookOpen,
  Code2,
  Download,
  GitCompare,
  Hammer,
  Keyboard,
  Play,
  RefreshCw,
  Save,
  Sparkles,
  Upload
} from "lucide-react";
import type {
  CompilerAvailability,
  EditorTheme,
  ExecutionLanguage,
  ExecutionMode,
  ExecutionProvider,
  SourceLanguage
} from "../types/studio";

interface ToolbarProps {
  sourceLanguage: SourceLanguage;
  onSourceLanguageChange: (language: SourceLanguage) => void;
  executionProvider: ExecutionProvider;
  onExecutionProviderChange: (provider: ExecutionProvider) => void;
  compilerAvailability: CompilerAvailability | null;
  theme: EditorTheme;
  onThemeChange: (theme: EditorTheme) => void;
  isDiffViewOpen: boolean;
  onToggleDiffView: () => void;
  isSwappedPanels: boolean;
  onSwapPanels: () => void;
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
  executionProvider,
  onExecutionProviderChange,
  compilerAvailability,
  theme,
  onThemeChange,
  isDiffViewOpen,
  onToggleDiffView,
  onSwapPanels,
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

  return (
    <header className="studio-toolbar">
      <div className="toolbar-group toolbar-brand">
        <Code2 size={20} className="brand-icon" />
        <span className="brand-title">RustC-Playground</span>
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
          className={`action-button ${isDiffViewOpen ? "active" : ""}`}
          onClick={onToggleDiffView}
          title="Toggle Diff View"
          data-testid="diff-toggle-button"
        >
          <GitCompare size={15} />
          <span>Diff</span>
        </button>

        <button
          type="button"
          className="action-button"
          onClick={onSwapPanels}
          title="Swap Left/Right Panels (Ctrl/Cmd + \)"
          data-testid="swap-panels-button"
        >
          <ArrowLeftRight size={15} />
          <span>Swap</span>
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

        <button
          type="button"
          className="action-button"
          onClick={onExportSource}
          title={`Export ${sourceLabel} file`}
        >
          <Download size={15} />
          <span>.{sourceLanguage}</span>
        </button>

        <button
          type="button"
          className="action-button"
          onClick={onExportRust}
          title="Export Rust file"
        >
          <Download size={15} />
          <span>.rs</span>
        </button>

        <button
          type="button"
          className="action-button"
          onClick={onSaveWorkspace}
          title="Save locally (Ctrl/Cmd + S)"
          data-testid="save-button"
        >
          <Save size={15} />
          <span>{saveStatusMessage ?? "Save"}</span>
        </button>

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
          <option value="local">Local (Auto-Fallback)</option>
          <option value="online">Online (JDoodle)</option>
        </select>

        <button
          type="button"
          className="exec-button compile-source"
          disabled={isExecuting}
          onClick={() => onExecute(sourceLanguage, "compile")}
          data-testid="compile-source-button"
        >
          <Hammer size={14} />
          <span>Compile {sourceLabel}</span>
        </button>
        <button
          type="button"
          className="exec-button run-source"
          disabled={isExecuting}
          onClick={() => onExecute(sourceLanguage, "run")}
          data-testid="run-source-button"
        >
          <Play size={14} />
          <span>Run {sourceLabel}</span>
        </button>
        <button
          type="button"
          className="exec-button compile-rust"
          disabled={isExecuting}
          onClick={() => onExecute("rust", "compile")}
          data-testid="compile-rust-button"
        >
          <Hammer size={14} />
          <span>Compile Rust</span>
        </button>
        <button
          type="button"
          className="exec-button run-rust"
          disabled={isExecuting}
          onClick={() => onExecute("rust", "run")}
          data-testid="run-rust-button"
        >
          <Play size={14} />
          <span>Run Rust</span>
        </button>
      </div>
    </header>
  );
};
