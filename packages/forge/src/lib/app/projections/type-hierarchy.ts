/**
 * Type hierarchy projection — type nodes with trait inheritance edges.
 *
 * Nodes from urdJson.types, edges from UrdTypeDef.traits.
 * Depends on urdJson.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

export const typeHierarchyProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.typeHierarchy',
  depends: ['urdJson'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { urdJson } = source;
    const types = urdJson.types;

    if (!types || Object.keys(types).length === 0) {
      return { nodes: [], edges: [] };
    }

    const nodes: ForgeGraphNode[] = Object.keys(types)
      .sort()
      .map((typeId) => ({
        id: typeId,
        label: typeId,
        kind: 'type' as const,
      }));

    const edges: ForgeGraphEdge[] = [];
    for (const [typeId, typeDef] of Object.entries(types)) {
      if (typeDef.traits) {
        for (const trait of typeDef.traits) {
          edges.push({
            id: `${typeId}→${trait}`,
            source: typeId,
            target: trait,
            kind: 'inheritance',
            label: 'extends',
          });
        }
      }
    }

    return { nodes, edges };
  },
};
