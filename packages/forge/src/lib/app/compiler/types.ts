/**
 * Compiler types — TypeScript representations of the Urd compiler output.
 *
 * Inner types (AST, SymbolTable, etc.) start as opaque interfaces with
 * minimal shape. They'll be refined incrementally as views consume them.
 */

// ===== Chunk system =====

export type ChunkName =
  | 'ast'
  | 'symbolTable'
  | 'factSet'
  | 'propertyDependencyIndex'
  | 'urdJson'
  | 'diagnostics';

export interface Chunk<T = unknown> {
  name: ChunkName;
  data: T;
  contentHash: string;
}

// ===== Compiler output =====

export interface PhaseTiming {
  phase: string;
  durationMs: number;
}

export interface WorldCounts {
  entities: number;
  locations: number;
  exits: number;
  properties: number;
  rules: number;
}

export interface OutputHeader {
  compileId: string;
  timestamp: number;
  durationMs: number;
  phaseTimings: PhaseTiming[];
  worldCounts: WorldCounts;
  inputFileCount: number;
}

export interface CompilerOutput {
  header: OutputHeader;
  chunks: Chunk[];
}

// ===== Resolved output (after cache de-duplication) =====

export interface ResolvedCompilerOutput {
  header: OutputHeader;
  ast: AST;
  symbolTable: SymbolTable;
  factSet: FactSet;
  propertyDependencyIndex: PropertyDependencyIndex;
  urdJson: UrdWorld;
  diagnostics: Diagnostic[];
}

// ===== Inner types — minimal shapes, refined later =====

export interface AST {
  nodes: unknown[];
}

export interface SymbolTableEntry {
  id: string;
  name: string;
  kind: string;
  file: string;
  line: number;
}

export interface SymbolTable {
  entries: SymbolTableEntry[];
}

export interface Fact {
  subject: string;
  predicate: string;
  object: string;
}

export interface FactSet {
  facts: Fact[];
}

export interface PropertyDependency {
  property: string;
  dependsOn: string[];
}

export interface PropertyDependencyIndex {
  dependencies: PropertyDependency[];
}

export interface UrdEntity {
  id: string;
  name: string;
  properties: Record<string, unknown>;
}

export interface UrdLocation {
  id: string;
  name: string;
  description: string;
  exits: UrdExit[];
}

export interface UrdExit {
  direction: string;
  target: string;
}

export interface UrdWorld {
  entities: UrdEntity[];
  locations: UrdLocation[];
}

export type DiagnosticSeverity = 'error' | 'warning' | 'info';

export interface DiagnosticSpan {
  file: string;
  startLine: number;
  startCol: number;
  endLine: number;
  endCol: number;
}

export interface Diagnostic {
  severity: DiagnosticSeverity;
  message: string;
  code: string;
  span: DiagnosticSpan | null;
}

export type AnalysisType = 'full' | 'incremental';

// ===== Compiler service interface =====

export interface CompilerService {
  compile(buffers: Record<string, string>): Promise<CompilerOutput>;
}
