/**
 * File path matching utility.
 *
 * The Rust compiler uses short filenames in spans (e.g., "main.urd.md")
 * while the Forge buffer map and editor tabs use full paths (e.g.,
 * "C:/Users/.../project/main.urd.md"). This utility handles both.
 */

/**
 * Check whether two file paths refer to the same file.
 * Handles the common case where one is a full path and the other is a
 * short filename from the compiler.
 */
export function filesMatch(fullPath: string, compilerPath: string): boolean {
  if (fullPath === compilerPath) return true;
  const norm = fullPath.replace(/\\/g, '/');
  return norm.endsWith('/' + compilerPath);
}

/**
 * Resolve a compiler short filename to the full buffer path.
 * Returns the matching full path, or the original compilerPath if no match found.
 */
export function resolveCompilerPath(compilerPath: string, knownPaths: string[]): string {
  const exact = knownPaths.find((p) => p === compilerPath);
  if (exact) return exact;
  const match = knownPaths.find((p) => filesMatch(p, compilerPath));
  return match ?? compilerPath;
}
