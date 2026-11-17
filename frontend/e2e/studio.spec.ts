import { expect, test } from "@playwright/test";

test.describe("RustC-Playground End-to-End Suite", () => {
  test("loads split-screen editors, pre-made examples, and real-time conversion", async ({
    page
  }) => {
    await page.goto("/");

    await expect(page.getByText("RustC-Playground", { exact: true })).toBeVisible();
    await expect(page.getByTestId("source-editor-pane")).toBeVisible();
    await expect(page.getByTestId("rust-editor-pane")).toBeVisible();

    await page.getByTestId("examples-button").click();
    await expect(page.getByTestId("examples-modal")).toBeVisible();
    await page.getByTestId("example-c-loops-printf").click();

    await expect(page.getByTestId("rust-editor-pane")).not.toBeEmpty();
    await page.getByTestId("output-style-select").selectOption("faithful");
    await expect(page.getByTestId("output-style-select")).toHaveValue("faithful");
    await expect(page.getByTestId("rust-editor-pane")).not.toBeEmpty();
    await page.getByTestId("output-style-select").selectOption("idiomatic");
    await expect(page.getByTestId("output-style-select")).toHaveValue("idiomatic");
    await expect(page.getByTestId("rust-editor-pane")).not.toBeEmpty();
  });

  test("toggles Diff View, swaps panels, changes theme, and saves locally", async ({
    page
  }) => {
    await page.goto("/");

    await page.getByTestId("diff-toggle-button").click();
    await expect(page.getByTestId("diff-viewer")).toBeVisible();
    await page.getByTestId("diff-toggle-button").click();

    await page.getByTestId("swap-panels-button").click();
    await page.getByTestId("theme-select").selectOption("midnight");
    await expect(page.getByTestId("studio-shell")).toHaveClass(/theme-midnight/);

    await page.getByTestId("save-button").click();
    await expect(page.getByTestId("save-button")).toContainText("Saved!");
  });

  test("catches misspelled #inlcude and undeclared fcvec identifier in diagnostics", async ({
    page
  }) => {
    await page.goto("/");

    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await page.keyboard.press("Control+a");
    await sourceEditorContent.fill(`#inlcude <iostream>
#include <vector>

int main(){
    std::vector<float> fvec;
    fvec.push_back(10.5);
    fvec.push_back(30.5);
    fvec.push_back(155.664);
    for(auto& vec: fcvec){
        std::cout << vec << std::endl;
    }
    return 0;
}`);

    await expect(page.getByTestId("rust-editor-pane")).toContainText(
      "// [Preprocessor] #inlcude <iostream>"
    );
    await expect(page.getByTestId("rust-editor-pane")).toContainText(
      "let mut fvec: Vec<f32> = Vec::new();"
    );

    await page.getByTestId("diagnostics-tab").click();
    const diagnosticsList = page.getByTestId("diagnostics-list");
    await expect(diagnosticsList).toContainText("#inlcude");
    await expect(diagnosticsList).toContainText("Did you mean '#include <iostream>'?");
    await expect(diagnosticsList).toContainText("fcvec");
    await expect(diagnosticsList).toContainText("Did you mean 'fvec'?");
  });

  test("runs valid float vector C++ and Rust locally and via JDoodle online API", async ({
    page
  }) => {
    await page.goto("/");

    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await sourceEditorContent.fill(`#include <iostream>
#include <vector>

int main() {
    std::vector<float> fvec;
    fvec.push_back(10.5);
    fvec.push_back(30.5);
    fvec.push_back(155.664);
    for (auto& vec : fvec) {
        std::cout << vec << std::endl;
    }
    return 0;
}`);

    await page.waitForTimeout(500);

    await page.getByTestId("provider-select").selectOption("local");
    await page.getByTestId("run-source-button").click();
    await expect(page.getByTestId("program-output")).toContainText("155.664", {
      timeout: 15000
    });
    await expect(page.getByTestId("used-provider-pill")).toContainText("local (");

    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("155.664", {
      timeout: 15000
    });
    await expect(page.getByTestId("used-provider-pill")).toContainText("local (");

    await page.getByTestId("provider-select").selectOption("online");
    await page.getByTestId("run-source-button").click();
    await expect(page.getByTestId("program-output")).toContainText("155.664", {
      timeout: 15000
    });
    await expect(page.getByTestId("used-provider-pill")).toContainText("online (");
  });

  test("converts and executes C forward declaration IsValidNumber locally in C and Rust", async ({
    page
  }) => {
    await page.goto("/");

    await page.getByTestId("examples-button").click();
    await expect(page.getByTestId("examples-modal")).toBeVisible();
    await page.getByTestId("example-c-base-validator").click();

    await expect(page.getByTestId("rust-editor-pane")).toContainText(
      "fn IsValidNumber(p_input: &str, base: i32) -> bool"
    );

    await page.getByTestId("provider-select").selectOption("local");
    await page.getByTestId("run-source-button").click();
    await expect(page.getByTestId("program-output")).toContainText(
      "The number 1A3F is valid in base 16",
      {
        timeout: 15000
      }
    );
    await expect(page.getByTestId("used-provider-pill")).toContainText("local (");

    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText(
      "The number 1A3F is valid in base 16",
      {
        timeout: 15000
      }
    );
    await expect(page.getByTestId("used-provider-pill")).toContainText("local (");
  });

  test("supports Manual Mode with Convert button and Very Complex BFS Graph example", async ({
    page
  }) => {
    await page.goto("/");

    await page.getByTestId("conversion-mode-select").selectOption("manual");
    await page.getByTestId("examples-button").click();
    await expect(page.getByTestId("examples-modal")).toBeVisible();
    await page.getByTestId("example-cpp-bfs-queue-graph").click();

    await page.getByTestId("convert-now-button").click();
    await expect(page.getByTestId("rust-editor-pane")).toContainText("VecDeque<i32>");

    await page.getByTestId("provider-select").selectOption("local");
    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("Visited: 4", {
      timeout: 15000
    });
  });

  test("explains std::vector<int> -> Vec<i32> ownership and iteration in output and Why Rust tab", async ({
    page
  }) => {
    await page.goto("/");

    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await page.keyboard.press("Control+a");
    await sourceEditorContent.fill(`#include <iostream>
#include <vector>

int main() {
    std::vector<int> values = {1, 2, 3};
    values.push_back(4);
    for (const auto& v : values) {
        std::cout << v << std::endl;
    }
    return 0;
}`);

    await expect(page.getByTestId("rust-editor-pane")).toContainText("Vec<i32>");
    await expect(page.getByTestId("rust-editor-pane")).not.toContainText(
      "// --- Rust Translation Notes"
    );
    await expect(page.getByTestId("rust-editor-pane")).not.toBeEmpty();
    await expect(page.getByTestId("explanations-tab")).toHaveCount(0);
    await expect(page.getByTestId("output-explanation-summary")).toHaveCount(0);
  });

  test("converts C malloc(sizeof(int)) and NULL check to Box<i32> and executes", async ({
    page
  }) => {
    await page.goto("/");

    await page.getByTestId("select-c-button").click();
    await page.getByTestId("conversion-mode-select").selectOption("manual");
    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await page.keyboard.press("Control+a");
    await sourceEditorContent.fill(`#include <stdlib.h>
#include <stdio.h>

int main(){
int *p = malloc(sizeof(int));
if (p != NULL) {
    *p = 100;
    printf("VALUE of p is %d\\n",*p);
    free(p);
}
  return 0;
}`);

    await page.getByTestId("convert-now-button").click();
    await expect(page.getByTestId("rust-editor-pane")).toContainText("malloc", {
      timeout: 15000
    });
    await page.getByTestId("output-style-select").selectOption("idiomatic");
    await expect(page.getByTestId("rust-editor-pane")).not.toBeEmpty();

    await page.getByTestId("provider-select").selectOption("local");
    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("VALUE of p is 100", {
      timeout: 15000
    });

    await page.getByTestId("output-style-select").selectOption("faithful");
    await expect(page.getByTestId("output-style-select")).toHaveValue("faithful");
    await expect(page.getByTestId("rust-editor-pane")).not.toBeEmpty();
    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("VALUE of p is 100", {
      timeout: 15000
    });
  });

  test("shows the engine and diagnostics panel for C++ exceptions", async ({
    page
  }) => {
    await page.goto("/");

    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await page.keyboard.press("Control+a");
    await sourceEditorContent.fill(`#include <iostream>
int main() {
    try {
        throw 1;
    } catch (int e) {
        std::cout << e << std::endl;
    }
    return 0;
}`);

    await expect(page.getByTestId("transpiler-engine-badge")).not.toBeEmpty();

    await page.getByTestId("diagnostics-tab").click();
    await expect(page.getByTestId("diagnostics-list")).toBeVisible();
  });
});
