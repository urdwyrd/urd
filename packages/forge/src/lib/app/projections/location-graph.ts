/**
 * Location graph projection — nodes and edges for graph visualisation.
 *
 * Depends on symbolTable, factSet.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

export interface GraphNode {
  id: string;
  name: string;
  description: string;
  exitCount: number;
}

export interface GraphEdge {
  from: string;
  to: string;
  direction: string;
}

export interface LocationGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export const locationGraphProjection: ProjectionDefinition<LocationGraph> = {
  id: 'urd.projection.locationGraph',
  depends: ['symbolTable', 'urdJson'],
  compute: (source: ResolvedCompilerOutput): LocationGraph => {
    const { urdJson } = source;

    const nodes: GraphNode[] = urdJson.locations.map((loc) => ({
      id: loc.id,
      name: loc.name,
      description: loc.description,
      exitCount: loc.exits.length,
    }));

    const edges: GraphEdge[] = urdJson.locations.flatMap((loc) =>
      loc.exits.map((exit) => ({
        from: loc.id,
        to: exit.target,
        direction: exit.direction,
      }))
    );

    return { nodes, edges };
  },
};
