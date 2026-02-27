/**
 * Location graph projection — enriched graph data for the Location Network Graph view.
 *
 * Nodes from urdJson.locations with diagnostic-driven flags (start, unreachable, isolated).
 * Edges from location exits.
 * Depends on urdJson, diagnostics.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

export const locationGraphProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.locationGraph',
  depends: ['urdJson', 'diagnostics'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { urdJson, diagnostics } = source;

    if (!urdJson.locations.length) {
      return { nodes: [], edges: [] };
    }

    const startId = urdJson.world?.start;

    // Track which locations participate in exits
    const exitParticipants = new Set<string>();
    const edges: ForgeGraphEdge[] = [];

    for (const loc of urdJson.locations) {
      for (const exit of loc.exits) {
        exitParticipants.add(loc.id);
        exitParticipants.add(exit.target);
        edges.push({
          id: `${loc.id}/${exit.direction}`,
          source: loc.id,
          target: exit.target,
          label: exit.direction,
          kind: 'normal',
        });
      }
    }

    const nodes: ForgeGraphNode[] = urdJson.locations.map((loc) => ({
      id: loc.id,
      label: loc.name || loc.id,
      kind: 'location' as const,
      flags: {
        start: loc.id === startId,
        unreachable: diagnostics.some(
          (d) => d.code === 'URD430' && d.message.includes(`'${loc.id}'`),
        ),
        isolated: !exitParticipants.has(loc.id),
      },
    }));

    return { nodes, edges };
  },
};
