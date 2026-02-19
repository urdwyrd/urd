/**
 * WASM compiler bridge — lazy-loads the Urd compiler module and provides
 * typed wrappers for the playground component.
 */

// --- Types ---

export interface CompileResult {
  success: boolean;
  world: string | null;
  diagnostics: Diagnostic[];
}

export interface ParseResult {
  success: boolean;
  diagnostics: Diagnostic[];
}

export interface Diagnostic {
  severity: 'error' | 'warning' | 'info';
  code: string;
  message: string;
  span: {
    file: string;
    start_line: number;
    start_col: number;
    end_line: number;
    end_col: number;
  };
}

// --- Module state ---

let wasmReady = false;
let wasmModule: typeof import('../../lib/wasm/urd_compiler') | null = null;

// --- Public API ---

/**
 * Lazy-load and initialise the WASM compiler module.
 * Safe to call multiple times — subsequent calls are no-ops.
 */
export async function initCompiler(): Promise<void> {
  if (wasmReady) return;

  const mod = await import('../../lib/wasm/urd_compiler');
  await mod.default(fetch('/wasm/urd_compiler_bg.wasm'));
  wasmModule = mod;
  wasmReady = true;
}

/**
 * Full five-phase compilation. Returns typed result.
 * Throws if the compiler is not initialised.
 */
export function compileSource(source: string): CompileResult {
  assertReady();
  try {
    const json = wasmModule!.compile_source(source);
    return JSON.parse(json) as CompileResult;
  } catch (e) {
    return {
      success: false,
      world: null,
      diagnostics: [{
        severity: 'error',
        code: 'WASM',
        message: `Compiler error: ${e instanceof Error ? e.message : String(e)}`,
        span: { file: 'playground.urd.md', start_line: 1, start_col: 1, end_line: 1, end_col: 1 },
      }],
    };
  }
}

/**
 * Parse-only (Phase 1). Returns typed result.
 * Throws if the compiler is not initialised.
 */
export function parseOnly(source: string): ParseResult {
  assertReady();
  try {
    const json = wasmModule!.parse_only(source);
    return JSON.parse(json) as ParseResult;
  } catch (e) {
    return {
      success: false,
      diagnostics: [{
        severity: 'error',
        code: 'WASM',
        message: `Parser error: ${e instanceof Error ? e.message : String(e)}`,
        span: { file: 'playground.urd.md', start_line: 1, start_col: 1, end_line: 1, end_col: 1 },
      }],
    };
  }
}

/**
 * Returns the compiler version string (e.g. "0.1.0").
 */
export function compilerVersion(): string {
  assertReady();
  return wasmModule!.compiler_version();
}

/**
 * Whether the WASM module has been loaded and initialised.
 */
export function isReady(): boolean {
  return wasmReady;
}

// --- Byte offset → character offset conversion ---

/**
 * Convert a byte-offset column (from the Rust compiler) to a character offset
 * for the given line text. Required for non-ASCII content where byte offsets
 * and character positions diverge.
 */
export function byteColToCharCol(lineText: string, byteCol: number): number {
  const encoder = new TextEncoder();
  let byteCount = 0;
  for (let i = 0; i < lineText.length; i++) {
    const charBytes = encoder.encode(lineText[i]).length;
    byteCount += charBytes;
    if (byteCount >= byteCol) return i + 1;
  }
  return lineText.length;
}

// --- Internal ---

function assertReady(): void {
  if (!wasmReady || !wasmModule) {
    throw new Error('Compiler not initialised. Call initCompiler() first.');
  }
}
