/**
 * Circular dependency projection — detects cycles in location exits
 * and section jumps using depth-first search.
 *
 * Finds cycles in two graphs:
 *  1. Location graph: edges from exit.from_location -> exit.to_location
 *  2. Section graph: edges from jump.from_section -> jump.target.id
 *
 * Depends on factSet, symbolTable.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface CycleEntry {
  path: string[];
  kind: 'location' | 'section';
}

export interface CircularDependencyResult {
  cycles: CycleEntry[];
  totalCycles: number;
}

/**
 * Detects cycles in a directed graph using iterative DFS.
 * Returns an array of cycle paths (each path is the cycle including
 * the repeated node at start and end).
 */
function findCycles(adjacency: Map<string, string[]>): string[][] {
  const cycles: string[][] = [];
  const visited = new Set<string>();
  const finished = new Set<string>();

  for (const startNode of adjacency.keys()) {
    if (finished.has(startNode)) continue;

    // Iterative DFS with explicit stack
    const stack: Array<{ node: string; pathIndex: number }> = [];
    const path: string[] = [];
    const onStack = new Set<string>();

    stack.push({ node: startNode, pathIndex: 0 });

    while (stack.length > 0) {
      const frame = stack[stack.length - 1];
      const { node } = frame;

      if (!visited.has(node)) {
        visited.add(node);
        onStack.add(node);
        path.push(node);
      }

      const neighbours = adjacency.get(node) ?? [];

      if (frame.pathIndex < neighbours.length) {
        const next = neighbours[frame.pathIndex];
        frame.pathIndex++;

        if (onStack.has(next)) {
          // Found a cycle — extract the cycle path
          const cycleStart = path.indexOf(next);
          if (cycleStart >= 0) {
            const cyclePath = [...path.slice(cycleStart), next];
            // Deduplicate: normalise so smallest element is first
            const minIdx = cyclePath.slice(0, -1).reduce(
              (min, _, i, arr) => (arr[i] < arr[min] ? i : min),
              0,
            );
            const normalised = [
              ...cyclePath.slice(minIdx, -1),
              ...cyclePath.slice(0, minIdx),
              cyclePath[minIdx],
            ];
            // Check we haven't already recorded this cycle
            const key = normalised.join('->');
            const isDuplicate = cycles.some(
              (c) => c.join('->') === key,
            );
            if (!isDuplicate) {
              cycles.push(normalised);
            }
          }
        } else if (!visited.has(next)) {
          stack.push({ node: next, pathIndex: 0 });
        }
      } else {
        // Backtrack
        stack.pop();
        onStack.delete(node);
        path.pop();
        finished.add(node);
      }
    }
  }

  return cycles;
}

export const circularDependencyProjection: ProjectionDefinition<CircularDependencyResult> = {
  id: 'urd.projection.circularDependency',
  depends: ['factSet', 'symbolTable'],
  compute: (source: ResolvedCompilerOutput): CircularDependencyResult => {
    const { factSet } = source;
    const cycles: CycleEntry[] = [];

    // Build location adjacency from exits
    const locationAdj = new Map<string, string[]>();
    for (const exit of factSet.exits) {
      if (!locationAdj.has(exit.from_location)) {
        locationAdj.set(exit.from_location, []);
      }
      locationAdj.get(exit.from_location)!.push(exit.to_location);
    }

    // Build section adjacency from jumps
    const sectionAdj = new Map<string, string[]>();
    for (const jump of factSet.jumps) {
      if (jump.target.id) {
        if (!sectionAdj.has(jump.from_section)) {
          sectionAdj.set(jump.from_section, []);
        }
        sectionAdj.get(jump.from_section)!.push(jump.target.id);
      }
    }

    // Detect location cycles
    for (const path of findCycles(locationAdj)) {
      cycles.push({ path, kind: 'location' });
    }

    // Detect section cycles
    for (const path of findCycles(sectionAdj)) {
      cycles.push({ path, kind: 'section' });
    }

    return {
      cycles,
      totalCycles: cycles.length,
    };
  },
};
