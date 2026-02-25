import type { CompileResult, DefinitionEntry } from './compiler-bridge';

export interface PlaygroundState {
  result: CompileResult | null;
  parsedWorld: Record<string, any> | null;
  definitionIndex: DefinitionEntry[] | null;
  compileTimeMs: number;
}

let current: PlaygroundState = {
  result: null,
  parsedWorld: null,
  definitionIndex: null,
  compileTimeMs: 0,
};

type Listener = () => void;
const listeners: Set<Listener> = new Set();

export function getState(): PlaygroundState {
  return current;
}

/**
 * Update shared state from a new compile result.
 * Retains stale parsedWorld and definitionIndex when a compile fails,
 * so autocomplete/hover/goto continue working during syntax errors.
 */
export function updateState(result: CompileResult, compileTimeMs?: number): void {
  const parsedWorld = result.world
    ? JSON.parse(result.world)
    : current.parsedWorld;
  const definitionIndex = result.definition_index?.definitions
    ?? current.definitionIndex;
  current = { result, parsedWorld, definitionIndex, compileTimeMs: compileTimeMs ?? current.compileTimeMs };
  listeners.forEach((fn) => fn());
}

export function subscribe(fn: Listener): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}
