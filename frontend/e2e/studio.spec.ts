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

  test("runs C++ and Rust code via JDoodle Live API", async ({ page }) => {
    await page.goto("/");

    await page.getByTestId("run-source-button").click();
    await expect(page.getByTestId("program-output")).toContainText(
      "RustC-Playground step 0",
      { timeout: 15000 }
    );

    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText(
      "RustC-Playground step 0",
      { timeout: 15000 }
    );
  });
});
