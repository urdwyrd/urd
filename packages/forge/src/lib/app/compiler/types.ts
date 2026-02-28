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
  | 'definitionIndex'
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
  definitionIndex: RichDefinitionIndex;
  urdJson: UrdWorld;
  /** Raw urdJson before normalisation — object-keyed, matching the compiler's output exactly. */
  rawUrdJson: Record<string, unknown> | null;
  diagnostics: Diagnostic[];
}

// ===== Inner types =====

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

// ===== FactSet — full compiler analysis IR =====

export interface FactSetSpan {
  file: string;
  start_line: number;
  start_col: number;
  end_line: number;
  end_col: number;
}

export interface FactSite {
  kind: 'choice' | 'exit' | 'rule';
  id: string;
}

export interface PropertyRead {
  site: FactSite;
  entity_type: string;
  property: string;
  operator: string;
  value_literal: string;
  value_kind: string;
  span: FactSetSpan;
}

export interface PropertyWrite {
  site: FactSite;
  entity_type: string;
  property: string;
  operator: string;
  value_expr: string;
  value_kind: string | null;
  span: FactSetSpan;
}

export interface ExitEdge {
  from_location: string;
  to_location: string;
  exit_name: string;
  is_conditional: boolean;
  guard_reads: number[];
  span: FactSetSpan;
}

export interface JumpTarget {
  kind: string;
  id?: string;
}

export interface JumpEdge {
  from_section: string;
  target: JumpTarget;
  span: FactSetSpan;
}

export interface ChoiceFact {
  section: string;
  choice_id: string;
  label: string;
  sticky: boolean;
  condition_reads: number[];
  effect_writes: number[];
  jump_indices: number[];
  span: FactSetSpan;
}

export interface RuleFact {
  rule_id: string;
  condition_reads: number[];
  effect_writes: number[];
  span: FactSetSpan;
}

export interface FactSet {
  reads: PropertyRead[];
  writes: PropertyWrite[];
  exits: ExitEdge[];
  jumps: JumpEdge[];
  choices: ChoiceFact[];
  rules: RuleFact[];
}

// Legacy alias for projections that use the old shape
export interface Fact {
  subject: string;
  predicate: string;
  object: string;
}

// ===== PropertyDependencyIndex — full compiler output =====

export interface PropertyDependencyEntry {
  entity_type: string;
  property: string;
  read_count: number;
  write_count: number;
  read_indices: number[];
  write_indices: number[];
  orphaned: string | null;
}

export interface PropertyDependencySummary {
  total_properties: number;
  total_reads: number;
  total_writes: number;
  read_never_written: number;
  written_never_read: number;
}

export interface PropertyDependencyIndex {
  properties: PropertyDependencyEntry[];
  summary: PropertyDependencySummary;
}

// Legacy alias
export interface PropertyDependency {
  property: string;
  dependsOn: string[];
}

// ===== Rich DefinitionIndex — from compiler's DefinitionIndex =====

export interface RichDefinitionSpan {
  file: string;
  start_line: number;
  start_col: number;
  end_line: number;
  end_col: number;
}

export interface RichDefinitionKind {
  kind: string;
  type_name?: string;
  property_type?: string;
  default?: string | null;
  local_name?: string;
  file_stem?: string;
  display_name?: string;
  from_location?: string;
  destination?: string;
  section_id?: string;
  label?: string;
}

export interface RichDefinitionEntry {
  key: string;
  span: RichDefinitionSpan;
  definition: RichDefinitionKind;
}

export interface RichDefinitionIndex {
  definitions: RichDefinitionEntry[];
  count: number;
}

// ===== World types =====

export interface UrdEntity {
  id: string;
  name: string;
  properties: Record<string, unknown>;
  type?: string;
  contains?: string[];
}

export interface UrdLocation {
  id: string;
  name: string;
  description: string;
  exits: UrdExit[];
  contains?: string[];
}

export interface UrdEffect {
  set?: string;
  to?: string | number | boolean;
  move?: string;
  destroy?: string;
  reveal?: string;
}

export interface UrdExit {
  direction: string;
  target: string;
  condition?: string;
  blocked_message?: string;
  effects?: UrdEffect[];
}

export interface UrdPropertyDef {
  type: string;
  default?: unknown;
  visibility?: string;
  description?: string;
  values?: string[];
  min?: number;
  max?: number;
}

export interface UrdTypeDef {
  traits?: string[];
  properties?: Record<string, UrdPropertyDef>;
}

export interface UrdWorldMeta {
  name?: string;
  version?: string;
  start?: string;
  entry?: string;
  seed?: string;
  description?: string;
  author?: string;
}

export interface UrdAction {
  description?: string;
  actor?: string;
  target?: string;
  target_type?: string;
  conditions?: string[] | { any: string[] };
  effects: UrdEffect[];
}

export interface UrdSequencePhase {
  id: string;
  prompt?: string;
  auto?: boolean;
  action?: string;
  actions?: string[];
  rule?: string;
  effects?: UrdEffect[];
  advance: string;
  condition?: string;
}

export interface UrdSequence {
  description?: string;
  phases: UrdSequencePhase[];
}

export interface UrdRuleSelect {
  from: string[];
  as: string;
  where?: string[];
}

export interface UrdRule {
  description?: string;
  actor?: string;
  trigger: string;
  conditions?: string[];
  select?: UrdRuleSelect;
  effects: UrdEffect[];
}

export interface UrdWorld {
  world?: UrdWorldMeta;
  types?: Record<string, UrdTypeDef>;
  entities: UrdEntity[];
  locations: UrdLocation[];
  /** Dialogue tree — keyed by dialogue node id. */
  dialogue?: Record<string, unknown>;
  /** Actions — keyed by action id. */
  actions?: Record<string, UrdAction>;
  /** Sequences — keyed by sequence id. */
  sequences?: Record<string, UrdSequence>;
  /** Rules — keyed by rule id. */
  rules?: Record<string, UrdRule>;
}

// ===== Diagnostics =====

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
  compile(buffers: Record<string, string>, entryFile?: string): Promise<CompilerOutput>;
}
