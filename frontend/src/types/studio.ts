export type SourceLanguage = "c" | "cpp";
export type ExecutionLanguage = "c" | "cpp" | "rust";
export type ExecutionMode = "compile" | "run";
export type ExecutionProvider = "local" | "online";
export type ConversionMode = "live" | "manual";
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
  transpilerEngine?: string;
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
  usedProvider: string;
}

export interface CompilerAvailability {
  cAvailable: boolean;
  cCompiler: string | null;
  cppAvailable: boolean;
  cppCompiler: string | null;
  rustAvailable: boolean;
  rustCompiler: string | null;
}

export interface CodeExample {
  identifier: string;
  title: string;
  category: string;
  difficulty: string;
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
  executionProvider?: ExecutionProvider;
  savedAtTimestamp: string;
}
