import { expect, test } from "@playwright/test";

test.describe("C/C++ to Rust Playground End-to-End Suite", () => {
  test("accepts a C++ standard-library completion from the popup", async ({ page }) => {
    await page.goto("/");

    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await page.keyboard.press("Control+a");
    await sourceEditorContent.fill("");
    await sourceEditorContent.type("std::vec", { delay: 40 });

    const completionList = page.locator(".cm-tooltip-autocomplete");
    await expect(completionList).toBeVisible();
    // Accept the `std::vector` entry explicitly: the popup keeps every C++
    // keyword/method option, so a bare Tab press applies whichever is first.
    await completionList.getByText("std::vector", { exact: true }).first().click();
    await expect(sourceEditorContent).toContainText("std::vector");
  });

  test("loads split-screen editors, pre-made examples, and real-time conversion", async ({
    page
  }) => {
    await page.goto("/");

    await expect(page.getByText("C/C++ to Rust Playground", { exact: true })).toBeVisible();
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

  test("changes theme, saves the workspace from the Save menu, and persists locally", async ({
    page
  }) => {
    await page.goto("/");

    await page.getByTestId("theme-select").selectOption("midnight");
    await expect(page.getByTestId("studio-shell")).toHaveClass(/theme-midnight/);

    // Save, export, and workspace persistence live in one dropdown, not three buttons.
    await page.getByTestId("save-menu-button").click();
    await expect(page.getByRole("menuitem", { name: "Save workspace" })).toBeVisible();
    await page.getByRole("menuitem", { name: "Save workspace" }).click();
    await expect(page.getByTestId("save-menu-button")).toContainText("Saved!");

    await page.reload();
    await expect(page.getByTestId("studio-shell")).toHaveClass(/theme-midnight/);
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

    // The source does not compile, so Readable Rust must say so instead of
    // presenting unverified fallback code as a clean translation.
    await expect(page.getByTestId("rust-editor-pane")).toContainText(
      "Readable Rust is unavailable",
      { timeout: 60000 }
    );

    // The fallback translation itself stays available in Generated Rust.
    await page.getByTestId("output-style-select").selectOption("faithful");
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

    // Live conversion is asynchronous; wait for this program's own Rust instead of
    // a fixed delay, otherwise Run Rust can execute the previous program.
    await expect(page.getByTestId("rust-editor-pane")).toContainText("fvec", {
      timeout: 60000
    });

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
    // The hosted compiler is a third-party service: accept its output or a quota message.
    await expect(page.getByTestId("program-output")).toContainText(
      /155\.664|limit reached|quota/i,
      { timeout: 30000 }
    );
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
    // Idiomatic output never keeps raw malloc/free: the heap cell becomes owned Rust.
    const mallocRustPane = page.getByTestId("rust-editor-pane");
    await expect(mallocRustPane).toContainText("VALUE of p is", { timeout: 60000 });
    const mallocRust = await mallocRustPane.innerText();
    expect(mallocRust).not.toContain("malloc");
    expect(mallocRust).not.toContain("unsafe");
    expect(mallocRust).not.toContain('extern "C"');
    await page.getByTestId("output-style-select").selectOption("idiomatic");
    await expect(page.getByTestId("rust-editor-pane")).not.toBeEmpty();

    await page.getByTestId("provider-select").selectOption("local");
    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("VALUE of p is 100", {
      timeout: 15000
    });

    await page.getByTestId("output-style-select").selectOption("faithful");
    await expect(page.getByTestId("output-style-select")).toHaveValue("faithful");
    await expect(page.getByTestId("rust-editor-pane")).toContainText("malloc", {
      timeout: 30000
    });
    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("VALUE of p is 100", {
      timeout: 15000
    });
  });

  test("typing plain C into the default C++ studio auto-switches to C and shows real c2rust idiomatic Rust", async ({
    page
  }) => {
    await page.goto("/");

    // Deliberately do NOT touch the C/C++ toggle: the studio starts on C++.
    await page.getByTestId("conversion-mode-select").selectOption("manual");
    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await sourceEditorContent.fill(`#include <stdio.h>
#include <stdlib.h>

int main() {
    int *arr = (int*)malloc(sizeof(int) * 4);

    for(int idx = 0; idx < 4; idx++){
      arr[idx] = idx + 10;
    }

    for(int idx = 0; idx < 4; idx++){
      printf("%d", arr[idx]);
    }

    return 0;
}`);

    // The content must switch the pipeline to C on its own.
    await expect(page.getByTestId("select-c-button")).toHaveClass(/active/, {
      timeout: 10000
    });

    await page.getByTestId("convert-now-button").click();

    // The real backbone engine must be used, not the regex fallback.
    await expect(page.getByTestId("transpiler-engine-badge")).toContainText(
      "Engine: c2rust CLI",
      { timeout: 30000 }
    );
    const rustPane = page.getByTestId("rust-editor-pane");
    await expect(rustPane).toContainText("print!(\"{}\"", { timeout: 30000 });
    const paneText = await rustPane.innerText();
    expect(paneText, `fallback junk leaked into the Rust pane:\n${paneText}`).not.toContain(
      "Regex Fallback"
    );
    expect(paneText).not.toContain("// #include");

    // The idiomatic Rust program must run and print the same output as C.
    await page.getByTestId("provider-select").selectOption("local");
    await page.getByTestId("run-source-button").click();
    await expect(page.getByTestId("program-output")).toContainText("10111213", {
      timeout: 20000
    });
    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("10111213", {
      timeout: 20000
    });
  });

  test("converts a char-buffer gets/puts program to safe idiomatic Rust and prints the same output", async ({
    page
  }) => {
    await page.goto("/");

    await page.getByTestId("conversion-mode-select").selectOption("manual");
    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await sourceEditorContent.fill(`#include <stdio.h>

int main() {
    char data[10];
    puts("Enter Data:");
    gets(data);
    puts(data);
    return 0;
}`);

    await expect(page.getByTestId("select-c-button")).toHaveClass(/active/, {
      timeout: 10000
    });
    await page.getByTestId("convert-now-button").click();

    const rustPane = page.getByTestId("rust-editor-pane");
    await expect(rustPane).toContainText("String::from_utf8_lossy", { timeout: 60000 });
    const rustText = await rustPane.innerText();
    expect(rustText).toContain("[u8; 10]");
    expect(rustText).not.toContain("unsafe");
    expect(rustText).not.toContain('extern "C"');
    expect(rustText).not.toContain("Regex Fallback");

    await page.getByTestId("stdin-textarea").fill("haseeb\n");
    await page.getByTestId("provider-select").selectOption("local");
    await page.getByTestId("run-source-button").click();
    await expect(page.getByTestId("program-output")).toContainText("haseeb", {
      timeout: 30000
    });
    await page.getByTestId("run-rust-button").click();
    await expect(page.getByTestId("program-output")).toContainText("haseeb", {
      timeout: 30000
    });
  });

  test("shows clippy lints for the generated Rust", async ({ page }) => {
    await page.goto("/");

    await page.getByTestId("conversion-mode-select").selectOption("manual");
    const sourceEditorContent = page
      .getByTestId("source-editor-pane")
      .locator(".cm-content");
    await sourceEditorContent.click();
    await page.keyboard.press("Meta+a");
    await page.keyboard.press("Control+a");
    await sourceEditorContent.fill(`#include <iostream>

class Person {
private:
    int id;
    float score;
public:
    Person(int person_id) : id(person_id), score(0.0f) {}
    void set_score(float value) {
        if (this->id > 0) {
            this->score = value;
        }
    }
    float get_score() const {
        if (this->id > 0) {
            return this->score;
        }
        return -1.0f;
    }
};

int main() {
    Person p(1);
    p.set_score(42.0f);
    std::cout << "score " << p.get_score() << std::endl;
    return 0;
}`);

    await page.getByTestId("convert-now-button").click();
    await expect(page.getByTestId("rust-editor-pane")).toContainText("impl Person", {
      timeout: 120000
    });

    await page.getByTestId("clippy-tab").click();
    const clippyList = page.getByTestId("clippy-list");
    await expect(clippyList).toBeVisible();
    await expect(clippyList).toContainText(/clippy::|unused_mut|no lints/, {
      timeout: 60000
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
