/**
 * Dialogue flow graph projection — sections as nodes, jumps as edges.
 *
 * Transforms factSet.jumps + factSet.choices into ForgeGraphData.
 * Choice labels attached via jump_indices correlation.
 * Depends on factSet, diagnostics.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput, ChoiceFact } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

/** Build a map from jump index → owning ChoiceFact. */
function buildJumpToChoiceMap(choices: ChoiceFact[]): Map<number, ChoiceFact> {
  const map = new Map<number, ChoiceFact>();
  for (const choice of choices) {
    for (const jumpIdx of choice.jump_indices) {
      map.set(jumpIdx, choice);
    }
  }
  return map;
}

export const dialogueGraphProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.dialogueGraph',
  depends: ['factSet', 'diagnostics'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { factSet, diagnostics } = source;

    if (!factSet.jumps.length && !factSet.choices.length) {
      return { nodes: [], edges: [] };
    }

    // Collect all unique section IDs
    const sectionIds = new Set<string>();
    for (const jump of factSet.jumps) {
      sectionIds.add(jump.from_section);
      if (jump.target.kind === 'section' && jump.target.id) {
        sectionIds.add(jump.target.id);
      }
    }
    for (const choice of factSet.choices) {
      sectionIds.add(choice.section);
    }

    // Section nodes
    const nodes: ForgeGraphNode[] = Array.from(sectionIds)
      .sort()
      .map((id) => ({
        id,
        label: id.split('/').pop() || id,
        kind: 'section' as const,
        flags: {
          unreachable: diagnostics.some(
            (d) => d.code === 'URD432' && d.message.includes(`'${id}'`),
          ),
        },
      }));

    // Terminal pseudo-nodes
    const hasEnd = factSet.jumps.some((j) => j.target.kind === 'end');
    if (hasEnd) {
      nodes.push({ id: '__end__', label: 'END', kind: 'terminal' as const });
    }

    const exitTargets = new Set<string>();
    for (const jump of factSet.jumps) {
      if (jump.target.kind === 'exit' && jump.target.id) {
        exitTargets.add(jump.target.id);
      }
    }
    for (const exitId of exitTargets) {
      nodes.push({
        id: `__exit_${exitId}`,
        label: `→ ${exitId}`,
        kind: 'terminal' as const,
      });
    }

    // Edges from JumpEdge tuples with choice correlation
    const jumpToChoice = buildJumpToChoiceMap(factSet.choices);

    const edges: ForgeGraphEdge[] = factSet.jumps.map((jump, i) => {
      const targetId =
        jump.target.kind === 'end'
          ? '__end__'
          : jump.target.kind === 'exit'
            ? `__exit_${jump.target.id}`
            : jump.target.id!;

      const matchingChoice = jumpToChoice.get(i) ?? null;

      let kind: ForgeGraphEdge['kind'] = 'normal';
      if (matchingChoice) {
        kind = matchingChoice.sticky ? 'choice_sticky' : 'choice_oneshot';
      } else if (jump.target.kind === 'end' || jump.target.kind === 'exit') {
        kind = 'terminal';
      }

      let label: string | undefined;
      if (matchingChoice) {
        const prefix = matchingChoice.sticky ? '+' : '•';
        label = `${prefix} ${matchingChoice.label}`;
      }

      return {
        id: `jump_${i}`,
        source: jump.from_section,
        target: targetId,
        label,
        kind,
      };
    });

    return { nodes, edges };
  },
};
