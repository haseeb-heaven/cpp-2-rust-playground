import React, { useState } from "react";
import { AlertTriangle, CheckCircle2, Clock, Cpu, ShieldCheck, Terminal, XCircle } from "lucide-react";
import type {
  ClippyLint,
  ConversionDiagnostic,
  ExecutionProvider,
  ExecutionResponse,
  LineMapping
} from "../types/studio";

interface OutputConsoleProps {
  stdinInput: string;
  onStdinChange: (value: string) => void;
  executionProvider: ExecutionProvider;
  executionResult: ExecutionResponse | null;
  lastExecutedLabel: string | null;
  isExecuting: boolean;
  conversionDiagnostics: ConversionDiagnostic[];
  clippyAnalysis?: {
    lints: ClippyLint[];
    error: string | null;
    checked: boolean;
    analyzedCode: string;
  } | null;
  currentRustCode?: string;
  lineMappings: LineMapping[];
  onSelectLineMapping: (sourceLine: number, rustStartLine: number, rustEndLine: number) => void;
  onSelectRustLine?: (lineNumber: number) => void;
}

type ConsoleTab = "output" | "diagnostics" | "clippy" | "mappings";

export const OutputConsole: React.FC<OutputConsoleProps> = ({
  stdinInput,
  onStdinChange,
  executionProvider,
  executionResult,
  lastExecutedLabel,
  isExecuting,
  conversionDiagnostics,
  clippyAnalysis = null,
  currentRustCode = "",
  lineMappings,
  onSelectLineMapping,
  onSelectRustLine
}) => {
  const [activeTab, setActiveTab] = useState<ConsoleTab>("output");

  const combinedDiagnostics = [
    ...conversionDiagnostics,
    ...(executionResult?.diagnostics ?? [])
  ];
  const clippyLints = clippyAnalysis?.lints ?? [];
  const clippyError = clippyAnalysis?.error ?? null;
  const clippyChecked = clippyAnalysis?.checked ?? false;
  // Lints belong to the exact Rust text that was analysed; a manual edit makes
  // their line numbers meaningless, so stop showing them until the next run.
  const clippyStale =
    clippyAnalysis !== null && currentRustCode !== clippyAnalysis.analyzedCode;
  const clippyErrors = clippyLints.filter((lint) => lint.level === "error").length;
  const clippyWarnings = clippyLints.filter((lint) => lint.level !== "error").length;
  const clippyMissing = clippyError !== null && /not installed|not found|no such file/i.test(clippyError);

  return (
    <section className="output-console" data-testid="output-console">
      <div className="console-tabs">
        <button
          type="button"
          className={`console-tab ${activeTab === "output" ? "active" : ""}`}
          onClick={() => setActiveTab("output")}
        >
          <Terminal size={14} />
          <span>Output &amp; Stdin</span>
        </button>

        <button
          type="button"
          className={`console-tab ${activeTab === "diagnostics" ? "active" : ""}`}
          onClick={() => setActiveTab("diagnostics")}
          data-testid="diagnostics-tab"
        >
          <AlertTriangle size={14} />
          <span>Diagnostics ({combinedDiagnostics.length})</span>
        </button>

        <button
          type="button"
          className={`console-tab ${activeTab === "clippy" ? "active" : ""}`}
          onClick={() => setActiveTab("clippy")}
          title="Rust clippy lints for the generated code"
          data-testid="clippy-tab"
        >
          <ShieldCheck size={14} />
          <span>Clippy ({clippyChecked && !clippyStale ? clippyLints.length : "?"})</span>
        </button>

        <button
          type="button"
          className={`console-tab ${activeTab === "mappings" ? "active" : ""}`}
          onClick={() => setActiveTab("mappings")}
          data-testid="mappings-tab"
        >
          <span>Line Mapping ({lineMappings.length})</span>
        </button>

        <div className="execution-metrics">
          <span className="provider-pill" data-testid="used-provider-pill">
            {executionResult?.usedProvider ??
              (executionProvider === "local" ? "local (ready)" : "online (JDoodle)")}
          </span>
          {executionResult && (
            <>
              <span className={`status-pill ${executionResult.success ? "success" : "failure"}`}>
                {executionResult.success ? (
                  <CheckCircle2 size={13} />
                ) : (
                  <XCircle size={13} />
                )}
                <span>
                  {lastExecutedLabel} · Exit {executionResult.statusCode}
                  {executionResult.timedOut ? " (Timed Out)" : ""}
                </span>
              </span>
              <span className="metric-item">
                <Clock size={13} />
                {executionResult.executionTimeMilliseconds} ms
              </span>
              {executionResult.cpuTimeSeconds && (
                <span className="metric-item">
                  <Cpu size={13} />
                  CPU {executionResult.cpuTimeSeconds}s
                </span>
              )}
              {executionResult.memoryUsed && (
                <span className="metric-item">Mem {executionResult.memoryUsed} KB</span>
              )}
            </>
          )}
        </div>
      </div>

      <div className="console-body">
        {activeTab === "output" && (
          <div className="output-split">
            <div className="stdin-panel">
              <label htmlFor="stdin-textarea" className="panel-label">
                Standard Input (stdin)
              </label>
              <textarea
                id="stdin-textarea"
                className="stdin-textarea"
                value={stdinInput}
                onChange={(event) => onStdinChange(event.target.value)}
                placeholder="Enter stdin lines here..."
                data-testid="stdin-textarea"
              />
            </div>

            <div className="stdout-panel">
              <div className="panel-label">
                Program Output {isExecuting ? "— Running..." : ""}
              </div>
              <pre className="output-pre" data-testid="program-output">
                {isExecuting
                  ? executionProvider === "local"
                    ? "Compiling and executing locally (gcc / g++ / rustc)..."
                    : "Compiling and executing via JDoodle Online API..."
                  : executionResult
                    ? executionResult.errorOutput
                      ? executionResult.errorOutput
                      : executionResult.output || "(Program exited with no output)"
                    : "Click Run to execute C, C++, or Rust code."}
              </pre>
            </div>
          </div>
        )}

        {activeTab === "diagnostics" && (
          <div className="diagnostics-list" data-testid="diagnostics-list">
            {combinedDiagnostics.length === 0 ? (
              <div className="empty-state">No conversion or compiler diagnostics.</div>
            ) : (
              combinedDiagnostics.map((diagnostic, index) => (
                <div
                  key={index}
                  className={`diagnostic-item severity-${diagnostic.severity}`}
                  onClick={() =>
                    onSelectLineMapping(
                      diagnostic.lineNumber,
                      diagnostic.lineNumber,
                      diagnostic.lineNumber
                    )
                  }
                >
                  <div className="diagnostic-header">
                    <span className="diagnostic-badge">
                      {diagnostic.severity.toUpperCase()} · {diagnostic.category}
                    </span>
                    <span className="diagnostic-line">Line {diagnostic.lineNumber}</span>
                  </div>
                  <div className="diagnostic-message">{diagnostic.message}</div>
                  {diagnostic.suggestedFix && (
                    <div className="diagnostic-fix">
                      Suggested fix: {diagnostic.suggestedFix}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === "clippy" && (
          <div className="diagnostics-list" data-testid="clippy-list">
            {clippyError !== null ? (
              <div className="empty-state" data-testid="clippy-error">
                {clippyMissing
                  ? `Clippy is not installed: ${clippyError}. Install it with \`rustup component add clippy\`.`
                  : `Clippy could not analyse this Rust: ${clippyError}`}
              </div>
            ) : clippyStale ? (
              <div className="empty-state" data-testid="clippy-stale">
                The Rust editor changed since the last Clippy run. Convert again to lint the
                current code.
              </div>
            ) : !clippyChecked ? (
              <div className="empty-state" data-testid="clippy-not-run">
                Clippy has not analysed this Rust yet. Convert the program to run it.
              </div>
            ) : clippyLints.length === 0 ? (
              <div className="empty-state" data-testid="clippy-clean">
                Clippy reports no lints for the generated Rust.
              </div>
            ) : (
              <>
                <div className="clippy-summary" data-testid="clippy-summary">
                  {clippyErrors} error{clippyErrors === 1 ? "" : "s"} · {clippyWarnings} warning
                  {clippyWarnings === 1 ? "" : "s"}
                </div>
                {clippyLints.map((lint, index) => (
                  <button
                    key={index}
                    type="button"
                    className={`diagnostic-item diagnostic-button severity-${
                      lint.level === "error" ? "error" : lint.level === "info" ? "info" : "warning"
                    }`}
                    onClick={() => {
                      if (onSelectRustLine) {
                        onSelectRustLine(lint.lineNumber);
                        return;
                      }
                      const mapping = lineMappings.find(
                        (item) =>
                          lint.lineNumber >= item.rustStartLine &&
                          lint.lineNumber <= item.rustEndLine
                      );
                      onSelectLineMapping(
                        mapping?.sourceLine ?? lint.lineNumber,
                        lint.lineNumber,
                        lint.lineNumber
                      );
                    }}
                  >
                    <div className="diagnostic-header">
                      <span className="diagnostic-badge">
                        {lint.level.toUpperCase()} · {lint.lint}
                      </span>
                      <span className="diagnostic-line">
                        Rust line {lint.lineNumber}:{lint.column}
                      </span>
                    </div>
                    <div className="diagnostic-message">{lint.message}</div>
                  </button>
                ))}
              </>
            )}
          </div>
        )}

        {activeTab === "mappings" && (
          <div className="mappings-table-wrapper">
            <table className="mappings-table">
              <thead>
                <tr>
                  <th>Source Line</th>
                  <th>Rust Lines</th>
                  <th>C/C++</th>
                  <th>Generated Rust</th>
                </tr>
              </thead>
              <tbody>
                {lineMappings.map((mapping, index) => (
                  <tr
                    key={index}
                    onClick={() =>
                      onSelectLineMapping(
                        mapping.sourceLine,
                        mapping.rustStartLine,
                        mapping.rustEndLine
                      )
                    }
                  >
                    <td>L{mapping.sourceLine}</td>
                    <td>
                      L{mapping.rustStartLine}
                      {mapping.rustEndLine > mapping.rustStartLine
                        ? `-L${mapping.rustEndLine}`
                        : ""}
                    </td>
                    <td>
                      <code>{mapping.sourceText}</code>
                    </td>
                    <td>
                      <code>{mapping.rustText}</code>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
};
