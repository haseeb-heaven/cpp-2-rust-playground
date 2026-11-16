export type SourceLanguage = "c" | "cpp";
export type ExecutionLanguage = "c" | "cpp" | "rust";
export type ExecutionMode = "compile" | "run";
export type EditorTheme = "dark" | "light" | "midnight";
export type DiagnosticSeverity = "error" | "warning" | "info";
export type ErrorCategory =
  | "syntax"
  | "compilation"
  | "runtime"
  | "conversion"
  | "network"
  | "input"
  | "memory"
  | "timeout";

export interface ConversionDiagnostic {
  lineNumber: number;
  severity: DiagnosticSeverity;
  category: ErrorCategory;
  message: string;
  suggestedFix: string | null;
}

export interface LineMapping {
  sourceLine: number;
  rustStartLine: number;
  rustEndLine: number;
  sourceText: string;
  rustText: string;
}

export interface ConversionResponse {
  generatedRustCode: string;
  lineMappings: LineMapping[];
  diagnostics: ConversionDiagnostic[];
  requiredImports: string[];
  conversionTimeMilliseconds: number;
}

export interface ExecutionResponse {
  success: boolean;
  output: string;
  errorOutput: string;
  statusCode: number;
  memoryUsed: string | null;
  cpuTimeSeconds: string | null;
  executionTimeMilliseconds: number;
  diagnostics: ConversionDiagnostic[];
  timedOut: boolean;
}

export interface CodeExample {
  identifier: string;
  title: string;
  category: string;
  sourceLanguage: SourceLanguage;
  description: string;
  sourceCode: string;
  defaultStdin: string;
}

export interface SavedWorkspaceState {
  sourceCode: string;
  sourceLanguage: SourceLanguage;
  rustCode: string;
  lastGeneratedRustCode: string;
  isRustManuallyEdited: boolean;
  stdinInput: string;
  theme: EditorTheme;
  isSwappedPanels: boolean;
  splitRatioPercent: number;
  savedAtTimestamp: string;
}
