import { describe, expect, it } from "vitest";
import { detectSourceLanguage } from "../utils/languageDetection";

const C_PROGRAM = `#include <stdio.h>
#include <stdlib.h>

int main() {
    int *arr = (int*)malloc(sizeof(int) * 4);
    for (int idx = 0; idx < 4; idx++) {
        arr[idx] = idx + 10;
    }
    printf("%d", arr[0]);
    return 0;
}`;

const CPP_PROGRAM = `#include <iostream>
#include <vector>
using namespace std;

int main() {
    vector<int> v = {1, 2, 3};
    for (int x : v) cout << x << endl;
    return 0;
}`;

describe("detectSourceLanguage", () => {
  it("detects plain C programs regardless of the current selection", () => {
    expect(detectSourceLanguage(C_PROGRAM, "cpp")).toBe("c");
    expect(detectSourceLanguage(C_PROGRAM, "c")).toBe("c");
  });

  it("detects C++ programs regardless of the current selection", () => {
    expect(detectSourceLanguage(CPP_PROGRAM, "c")).toBe("cpp");
    expect(detectSourceLanguage(CPP_PROGRAM, "cpp")).toBe("cpp");
  });

  it("keeps the current language for ambiguous or empty sources", () => {
    expect(detectSourceLanguage("int main() { return 0; }", "c")).toBe("c");
    expect(detectSourceLanguage("int main() { return 0; }", "cpp")).toBe("cpp");
    expect(detectSourceLanguage("", "c")).toBe("c");
    expect(detectSourceLanguage("   \n  ", "cpp")).toBe("cpp");
  });

  it("does not let comments or strings fake the language", () => {
    const cWithCppComment = `#include <stdio.h>
// this mentions cout and std::vector in a comment
int main(void) { printf("cin class new"); return 0; }`;
    expect(detectSourceLanguage(cWithCppComment, "c")).toBe("c");

    const cppWithCWords = `#include <iostream>
int main() { std::cout << "printf malloc sizeof" << std::endl; return 0; }`;
    expect(detectSourceLanguage(cppWithCWords, "cpp")).toBe("cpp");
  });

  it("prefers C++ markers when both languages appear", () => {
    const mixed = `#include <stdio.h>
#include <iostream>
int main() { std::cout << 1; return 0; }`;
    expect(detectSourceLanguage(mixed, "c")).toBe("cpp");
  });
});
