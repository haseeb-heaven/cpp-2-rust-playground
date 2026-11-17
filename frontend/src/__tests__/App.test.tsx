import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import App from "../App";

describe("App component", () => {
  beforeEach(() => {
    window.localStorage.clear();
    vi.stubGlobal(
      "fetch",
      vi.fn((input: RequestInfo | URL) => {
        const url = typeof input === "string" ? input : input.toString();
        if (url.includes("/api/compilers")) {
          return Promise.resolve(
            new Response(
              JSON.stringify({
                cAvailable: true,
                cCompiler: "gcc",
                cppAvailable: true,
                cppCompiler: "g++",
                rustAvailable: true,
                rustCompiler: "rustc"
              }),
              { status: 200 }
            )
          );
        }
        if (url.includes("/api/examples")) {
          return Promise.resolve(
            new Response(
              JSON.stringify([
                {
                  identifier: "cpp-hello",
                  title: "Hello C++",
                  category: "Basics",
                  sourceLanguage: "cpp",
                  description: "Basic hello",
                  sourceCode: "int main() { return 0; }",
                  defaultStdin: ""
                }
              ]),
              { status: 200 }
            )
          );
        }
        if (url.includes("/api/convert")) {
          return Promise.resolve(
            new Response(
              JSON.stringify({
                generatedRustCode:
                  "fn main() {\n    println!(\"ok\");\n}\n\n// --- Rust Translation Notes ---\n// [Dynamic Array] std::vector<int> -> Vec<i32>",
                lineMappings: [],
                diagnostics: [],
                requiredImports: [],
                conversionTimeMilliseconds: 2,
                transpilerEngine: "Built-in Regex C++ Fallback (cpp2rust CLI not executed)",
                usedFallback: true,
                fallbackReason: "External cpp2rust CLI binary is not installed; used built-in regex-based fallback translator.",
                unsupportedConstructs: [],
                postProcessing: {
                  appliedPasses: ["rustfmt"],
                  rejectedPasses: [],
                  formattedWithRustfmt: true,
                  compilerVerified: true,
                  unsafeBlocksRemoved: 0,
                  redundantMutRemoved: 0,
                  redundantCastsRemoved: 0
                },
                explanations: [
                  {
                    construct: "Dynamic Array (std::vector → Vec)",
                    cppPattern: "std::vector<int>",
                    rustEquivalent: "Vec<i32>",
                    whyChosen: "Vec<i32> is Rust's growable contiguous heap array.",
                    ownershipAndIteration: "Iterate immutably via &vec or mutably via &mut vec."
                  }
                ]
              }),
              { status: 200 }
            )
          );
        }
        return Promise.resolve(
          new Response(
            JSON.stringify({
              success: true,
              output: "Hello from Rust!\n",
              errorOutput: "",
              statusCode: 200,
              memoryUsed: "1024",
              cpuTimeSeconds: "0.01",
              executionTimeMilliseconds: 120,
              diagnostics: [],
              timedOut: false,
              usedProvider: "local (rustc)"
            }),
            { status: 200 }
          )
        );
      })
    );
  });

  it("renders RustC-Playground toolbar, editors, and console", () => {
    render(<App />);
    expect(screen.getByText("RustC-Playground")).toBeInTheDocument();
    expect(screen.getByTestId("source-editor-pane")).toBeInTheDocument();
    expect(screen.getByTestId("rust-editor-pane")).toBeInTheDocument();
    expect(screen.getByTestId("output-console")).toBeInTheDocument();
  });

  it("toggles Diff View and swaps panels", () => {
    render(<App />);
    const diffButton = screen.getByTestId("diff-toggle-button");
    fireEvent.click(diffButton);
    expect(screen.getByTestId("diff-viewer")).toBeInTheDocument();
    fireEvent.click(diffButton);
    expect(screen.queryByTestId("diff-viewer")).not.toBeInTheDocument();

    const swapButton = screen.getByTestId("swap-panels-button");
    fireEvent.click(swapButton);
    expect(screen.getByTestId("studio-shell")).toBeInTheDocument();
  });

  it("executes Rust code and displays output in console", async () => {
    render(<App />);
    const runRustButton = screen.getByTestId("run-rust-button");
    fireEvent.click(runRustButton);

    await waitFor(() => {
      expect(screen.getByTestId("program-output")).toHaveTextContent("Hello from Rust!");
      expect(screen.getByTestId("used-provider-pill")).toHaveTextContent("local (rustc)");
    });
  });

  it("allows switching between local and online compiler provider", () => {
    render(<App />);
    const providerSelect = screen.getByTestId("provider-select") as HTMLSelectElement;
    expect(providerSelect.value).toBe("local");
    fireEvent.change(providerSelect, { target: { value: "online" } });
    expect(providerSelect.value).toBe("online");
  });

  it("allows switching between Live Mode and Manual Mode and clicking Convert", async () => {
    render(<App />);
    const modeSelect = screen.getByTestId("conversion-mode-select") as HTMLSelectElement;
    expect(modeSelect.value).toBe("live");
    fireEvent.change(modeSelect, { target: { value: "manual" } });
    expect(modeSelect.value).toBe("manual");

    const convertButton = screen.getByTestId("convert-now-button");
    fireEvent.click(convertButton);
    await waitFor(() => {
      expect(screen.getByTestId("rust-editor-pane")).toHaveTextContent("println!(\"ok\")");
      expect(screen.getByTestId("rust-editor-pane")).toHaveTextContent("Vec<i32>");
      expect(screen.getByTestId("transpiler-engine-badge")).toHaveTextContent("Regex Fallback");
    });
    expect(screen.queryByTestId("explanations-tab")).not.toBeInTheDocument();
    expect(screen.queryByTestId("output-explanation-summary")).not.toBeInTheDocument();
  });
});
