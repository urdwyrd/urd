/**
 * Containment tree projection — world → locations → entities hierarchy.
 *
 * Uses containment edge kind. Depends on urdJson.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

export const containmentTreeProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.containmentTree',
  depends: ['urdJson'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { urdJson } = source;
    const nodes: ForgeGraphNode[] = [];
    const edges: ForgeGraphEdge[] = [];

    if (!urdJson.locations.length && !urdJson.entities.length) {
      return { nodes, edges };
    }

    // World root node
    const worldName = urdJson.world?.name || 'World';
    nodes.push({
      id: '__world__',
      label: worldName,
      kind: 'entity' as const,
      flags: { start: true },
    });

    // Location nodes
    for (const loc of urdJson.locations) {
      nodes.push({
        id: `loc:${loc.id}`,
        label: loc.name || loc.id,
        kind: 'location' as const,
      });
      edges.push({
        id: `world→${loc.id}`,
        source: '__world__',
        target: `loc:${loc.id}`,
        kind: 'containment',
      });
    }

    // Entity nodes — place under their containing location or world root
    const entityLocationMap = new Map<string, string>();
    for (const loc of urdJson.locations) {
      if (loc.contains) {
        for (const entityId of loc.contains) {
          entityLocationMap.set(entityId, loc.id);
        }
      }
    }

    for (const entity of urdJson.entities) {
      nodes.push({
        id: `ent:${entity.id}`,
        label: entity.name || entity.id,
        kind: 'entity' as const,
      });

      const containingLoc = entityLocationMap.get(entity.id);
      const parentId = containingLoc ? `loc:${containingLoc}` : '__world__';
      edges.push({
        id: `${parentId}→ent:${entity.id}`,
        source: parentId,
        target: `ent:${entity.id}`,
        kind: 'containment',
      });

      // Nested containment (entity contains other entities)
      if (entity.contains) {
        for (const childId of entity.contains) {
          edges.push({
            id: `ent:${entity.id}→ent:${childId}`,
            source: `ent:${entity.id}`,
            target: `ent:${childId}`,
            kind: 'containment',
          });
        }
      }
    }

    return { nodes, edges };
  },
};
