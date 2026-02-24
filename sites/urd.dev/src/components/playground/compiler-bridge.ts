/**
 * WASM compiler bridge — lazy-loads the Urd compiler module and provides
 * typed wrappers for the playground component.
 */

// Injected by Vite `define` at build time (compiler version for cache-busting).
declare const __WASM_CACHE_BUST__: string;

// --- Types ---

export interface FactSite {
  kind: 'choice' | 'exit' | 'rule';
  id: string;
}

export interface FactSpan {
  file: string;
  start_line: number;
  start_col: number;
  end_line: number;
  end_col: number;
}

export interface PropertyRead {
  site: FactSite;
  entity_type: string;
  property: string;
  operator: string;
  value_literal: string;
  value_kind: string;
  span: FactSpan;
}

export interface PropertyWrite {
  site: FactSite;
  entity_type: string;
  property: string;
  operator: string;
  value_expr: string;
  value_kind: string | null;
  span: FactSpan;
}

export interface ExitEdge {
  from_location: string;
  to_location: string;
  exit_name: string;
  is_conditional: boolean;
  guard_reads: number[];
  span: FactSpan;
}

export interface JumpEdge {
  from_section: string;
  target: { kind: 'section' | 'exit' | 'end'; id?: string };
  span: FactSpan;
}

export interface ChoiceFact {
  section: string;
  choice_id: string;
  label: string;
  sticky: boolean;
  condition_reads: number[];
  effect_writes: number[];
  jump_indices: number[];
  span: FactSpan;
}

export interface RuleFact {
  rule_id: string;
  condition_reads: number[];
  effect_writes: number[];
  span: FactSpan;
}

export interface FactSet {
  reads: PropertyRead[];
  writes: PropertyWrite[];
  exits: ExitEdge[];
  jumps: JumpEdge[];
  choices: ChoiceFact[];
  rules: RuleFact[];
}

export interface PropertyEntry {
  entity_type: string;
  property: string;
  read_count: number;
  write_count: number;
  read_indices: number[];
  write_indices: number[];
  orphaned: 'read_never_written' | 'written_never_read' | null;
}

export interface PropertyIndexSummary {
  total_properties: number;
  total_reads: number;
  total_writes: number;
  read_never_written: number;
  written_never_read: number;
}

export interface PropertyIndex {
  properties: PropertyEntry[];
  summary: PropertyIndexSummary;
}

export interface CompileResult {
  success: boolean;
  world: string | null;
  diagnostics: Diagnostic[];
  facts: FactSet | null;
  property_index: PropertyIndex | null;
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
  // Cache-bust: the WASM filename is fixed, so append a build-time hash
  // to force browsers to fetch the new binary after each deploy.
  const bustParam = __WASM_CACHE_BUST__ ? `?v=${__WASM_CACHE_BUST__}` : '';
  await mod.default(fetch(`/wasm/urd_compiler_bg.wasm${bustParam}`));
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
      facts: null,
      property_index: null,
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
