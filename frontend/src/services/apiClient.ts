import type {
  CodeExample,
  ConversionResponse,
  ExecutionLanguage,
  ExecutionMode,
  ExecutionResponse,
  SourceLanguage
} from "../types/studio";

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "";

export async function convertSourceCode(
  sourceCode: string,
  sourceLanguage: SourceLanguage,
  abortSignal?: AbortSignal
): Promise<ConversionResponse> {
  const response = await fetch(`${API_BASE_URL}/api/convert`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ sourceCode, sourceLanguage }),
    signal: abortSignal
  });

  if (!response.ok) {
    throw new Error(`Conversion failed with HTTP status ${response.status}`);
  }

  return (await response.json()) as ConversionResponse;
}

export async function executeProgramCode(
  code: string,
  language: ExecutionLanguage,
  mode: ExecutionMode,
  stdinInput: string
): Promise<ExecutionResponse> {
  const response = await fetch(`${API_BASE_URL}/api/execute`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code, language, mode, stdinInput })
  });

  if (!response.ok) {
    throw new Error(`Execution request failed with HTTP status ${response.status}`);
  }

  return (await response.json()) as ExecutionResponse;
}

export async function fetchPremadeExamples(): Promise<CodeExample[]> {
  const response = await fetch(`${API_BASE_URL}/api/examples`);
  if (!response.ok) {
    throw new Error(`Failed to fetch examples (${response.status})`);
  }
  return (await response.json()) as CodeExample[];
}
