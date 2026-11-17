import type { SourceLanguage } from "../types/studio";

/**
 * Content-based C vs C++ detection.
 *
 * The studio must never send plain C through the C++ engine (or the other way
 * round): the backbones are language-specific, so a wrong guess silently
 * degrades every conversion for that program. Detection only switches language
 * when the source contains decisive markers; ambiguous sources keep the
 * current selection, and an explicit toolbar choice still wins until the
 * content proves otherwise.
 */

const CPP_ONLY_HEADERS = [
  "iostream",
  "vector",
  "list",
  "map",
  "set",
  "unordered_map",
  "unordered_set",
  "deque",
  "queue",
  "stack",
  "algorithm",
  "memory",
  "functional",
  "utility",
  "tuple",
  "bitset",
  "numeric",
  "sstream",
  "fstream",
  "iomanip",
  "string",
  "array",
  "complex",
  "random",
  "stdexcept",
  "cstring",
  "cstdlib",
  "cstdio",
  "cmath",
  "cassert",
];

const C_ONLY_HEADERS = [
  "stdio.h",
  "stdlib.h",
  "string.h",
  "stdint.h",
  "stdbool.h",
  "stddef.h",
  "math.h",
  "limits.h",
  "float.h",
  "ctype.h",
  "errno.h",
  "assert.h",
  "signal.h",
  "stdarg.h",
  "iso646.h",
  "wchar.h",
  "stdalign.h",
  "stdnoreturn.h",
  "inttypes.h",
  "time.h",
  "fenv.h",
];

const CPP_TOKEN_REGEXES: RegExp[] = [
  /\bstd::/,
  /\busing\s+namespace\b/,
  /\b(cout|cin|cerr|clog|endl|wcout|wcin)\b/,
  /\b(nullptr|template|namespace|operator|typename|virtual|override|friend|throw|try|catch)\b/,
  /\b(public|private|protected)\s*:/,
  /\bclass\b/,
  /\bnew\s+\w/,
  /\bdelete\s+\w/,
];

const C_TOKEN_REGEXES: RegExp[] = [
  /\btypedef\s+struct\b/,
  /#\s*define\b/,
  /\b_Static_assert\b/,
  /\b(malloc|calloc|realloc)\s*\(/,
  /\bfree\s*\(/,
  /\bprintf\s*\(/,
  /\bscanf\s*\(/,
  /\bputs\s*\(/,
  /\bputchar\s*\(/,
  /\bsizeof\b/,
  /\bNULL\b/,
];

function stripCommentsAndStrings(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, " ")
    .replace(/\/\/[^\n]*/g, " ")
    .replace(/"(?:[^"\\\n]|\\.)*"/g, " ")
    .replace(/'(?:[^'\\\n]|\\.)*'/g, " ");
}

export function detectSourceLanguage(
  source: string,
  current: SourceLanguage
): SourceLanguage {
  const code = stripCommentsAndStrings(source);
  if (code.trim().length === 0) {
    return current;
  }

  const cppHeaderIncludes = CPP_ONLY_HEADERS.some((header) =>
    new RegExp(`#\\s*include\\s*[<"]${header}[>"]`).test(code)
  );
  const cppMarker =
    cppHeaderIncludes ||
    CPP_TOKEN_REGEXES.some((pattern) => pattern.test(code));

  if (cppMarker) {
    return "cpp";
  }

  const cHeaderIncludes = C_ONLY_HEADERS.some((header) =>
    new RegExp(`#\\s*include\\s*[<"]${header}[>"]`).test(code)
  );
  const cMarker = cHeaderIncludes || C_TOKEN_REGEXES.some((pattern) => pattern.test(code));

  if (cMarker) {
    return "c";
  }

  return current;
}
