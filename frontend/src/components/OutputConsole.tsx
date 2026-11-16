import React, { useState } from "react";
import { AlertTriangle, CheckCircle2, Clock, Cpu, Terminal, XCircle } from "lucide-react";
import type {
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
  lineMappings: LineMapping[];
  onSelectLineMapping: (sourceLine: number, rustStartLine: number, rustEndLine: number) => void;
}

type ConsoleTab = "output" | "diagnostics" | "mappings";

export const OutputConsole: React.FC<OutputConsoleProps> = ({
  stdinInput,
  onStdinChange,
  executionProvider,
  executionResult,
  lastExecutedLabel,
  isExecuting,
  conversionDiagnostics,
  lineMappings,
  onSelectLineMapping
}) => {
  const [activeTab, setActiveTab] = useState<ConsoleTab>("output");

  const combinedDiagnostics = [
    ...conversionDiagnostics,
    ...(executionResult?.diagnostics ?? [])
  ];

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
                    : "Click Compile or Run to execute C, C++, or Rust code."}
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
