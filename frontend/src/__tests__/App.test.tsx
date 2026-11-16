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
                generatedRustCode: "fn main() {\n    println!(\"ok\");\n}",
                lineMappings: [],
                diagnostics: [],
                requiredImports: [],
                conversionTimeMilliseconds: 2
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
              timedOut: false
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
    });
  });
});
