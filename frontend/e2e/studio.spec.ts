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

    await expect(page.getByTestId("rust-editor-pane")).toContainText("for index in 0..limit");
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

    await page.waitForTimeout(500);

    const rustEditorText = await page.getByTestId("rust-editor-pane").innerText();
    expect(rustEditorText).toContain("// [Preprocessor] #inlcude <iostream>");
    expect(rustEditorText).toContain("let mut fvec: Vec<f32> = Vec::new();");

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
    await expect(page.getByTestId("program-output")).toContainText("1A3F base 16: valid", {
      timeout: 15000
    });
    await expect(page.getByTestId("used-provider-pill")).toContainText("local (");

    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("19Z base 10: invalid", {
      timeout: 15000
    });
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
});
