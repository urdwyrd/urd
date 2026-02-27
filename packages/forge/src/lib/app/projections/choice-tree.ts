/**
 * Choice tree projection — sections and their choices as a tree structure.
 *
 * Nodes: sections (kind='section') + individual choices (kind='property').
 * Edges: section->choice containment (kind='containment'),
 *        choice->target section via jump_indices (kind='normal' or
 *        'choice_sticky'/'choice_oneshot').
 * Depends on factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

export const choiceTreeProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.choiceTree',
  depends: ['factSet'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { factSet } = source;

    if (!factSet.choices.length) {
      return { nodes: [], edges: [] };
    }

    const nodeIds = new Set<string>();
    const nodes: ForgeGraphNode[] = [];
    const edges: ForgeGraphEdge[] = [];

    function ensureSection(id: string): void {
      if (!nodeIds.has(id)) {
        nodeIds.add(id);
        nodes.push({
          id,
          label: id.split('/').pop() || id,
          kind: 'section' as const,
        });
      }
    }

    for (const choice of factSet.choices) {
      // Ensure the owning section exists
      ensureSection(choice.section);

      // Add the choice node
      const choiceId = `choice:${choice.choice_id}`;
      if (!nodeIds.has(choiceId)) {
        nodeIds.add(choiceId);
        const prefix = choice.sticky ? '+' : '\u2022';
        nodes.push({
          id: choiceId,
          label: `${prefix} ${choice.label}`,
          kind: 'property' as const,
        });
      }

      // Containment edge: section -> choice
      edges.push({
        id: `contain_${choice.section}_${choice.choice_id}`,
        source: choice.section,
        target: choiceId,
        kind: 'containment',
      });

      // Jump edges: choice -> target sections
      for (const jumpIdx of choice.jump_indices) {
        const jump = factSet.jumps[jumpIdx];
        if (!jump) continue;

        let targetId: string;
        if (jump.target.kind === 'section' && jump.target.id) {
          targetId = jump.target.id;
          ensureSection(targetId);
        } else if (jump.target.kind === 'end') {
          targetId = '__end__';
          if (!nodeIds.has(targetId)) {
            nodeIds.add(targetId);
            nodes.push({ id: targetId, label: 'END', kind: 'terminal' as const });
          }
        } else if (jump.target.kind === 'exit' && jump.target.id) {
          targetId = `__exit_${jump.target.id}`;
          if (!nodeIds.has(targetId)) {
            nodeIds.add(targetId);
            nodes.push({
              id: targetId,
              label: `-> ${jump.target.id}`,
              kind: 'terminal' as const,
            });
          }
        } else {
          continue;
        }

        const kind = choice.sticky ? 'choice_sticky' : 'choice_oneshot';
        edges.push({
          id: `jump_${choice.choice_id}_${jumpIdx}`,
          source: choiceId,
          target: targetId,
          label: choice.label,
          kind,
        });
      }
    }

    return { nodes, edges };
  },
};
