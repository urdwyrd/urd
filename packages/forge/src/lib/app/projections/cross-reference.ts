/**
 * Cross-reference graph projection — symbols and their references.
 *
 * Nodes: locations, entity types, entities.
 * Edges: references derived from factSet (exits reference locations,
 * reads/writes reference entity types).
 * Depends on factSet, urdJson.
 */

import type { ProjectionDefinition } from './ProjectionRegistry';
import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';
import type { ForgeGraphData, ForgeGraphNode, ForgeGraphEdge } from '$lib/app/views/graphs/_shared/graph-types';

export const crossReferenceProjection: ProjectionDefinition<ForgeGraphData> = {
  id: 'urd.projection.crossReference',
  depends: ['factSet', 'urdJson'],
  compute: (source: ResolvedCompilerOutput): ForgeGraphData => {
    const { factSet, urdJson } = source;
    const nodes: ForgeGraphNode[] = [];
    const edges: ForgeGraphEdge[] = [];
    const nodeIds = new Set<string>();

    // Add location nodes
    for (const loc of urdJson.locations) {
      const id = `loc:${loc.id}`;
      if (!nodeIds.has(id)) {
        nodeIds.add(id);
        nodes.push({
          id,
          label: loc.name || loc.id,
          kind: 'location' as const,
        });
      }
    }

    // Add type nodes
    if (urdJson.types) {
      for (const typeId of Object.keys(urdJson.types)) {
        const id = `type:${typeId}`;
        if (!nodeIds.has(id)) {
          nodeIds.add(id);
          nodes.push({
            id,
            label: typeId,
            kind: 'type' as const,
          });
        }
      }
    }

    // Exit edges: location → location
    const edgeIds = new Set<string>();
    for (const exit of factSet.exits) {
      const fromId = `loc:${exit.from_location}`;
      const toId = `loc:${exit.to_location}`;
      const edgeId = `exit:${fromId}→${toId}`;
      if (!edgeIds.has(edgeId)) {
        edgeIds.add(edgeId);
        edges.push({
          id: edgeId,
          source: fromId,
          target: toId,
          kind: 'reference',
          label: 'exit',
        });
      }
    }

    // Read/write edges: entity type references from FactSite
    for (const read of factSet.reads) {
      const typeId = `type:${read.entity_type}`;
      // FactSite has { kind, id } — kind is 'choice'|'exit'|'rule'
      const sitePrefix = read.site.kind === 'exit' ? 'loc' : 'sec';
      const siteId = `${sitePrefix}:${read.site.id}`;

      if (nodeIds.has(typeId)) {
        // Ensure site node exists
        if (!nodeIds.has(siteId)) {
          nodeIds.add(siteId);
          nodes.push({
            id: siteId,
            label: read.site.id,
            kind: sitePrefix === 'loc' ? 'location' as const : 'section' as const,
          });
        }

        const edgeId = `read:${siteId}→${typeId}:${read.property}`;
        if (!edgeIds.has(edgeId)) {
          edgeIds.add(edgeId);
          edges.push({
            id: edgeId,
            source: siteId,
            target: typeId,
            kind: 'reference',
            label: `reads .${read.property}`,
          });
        }
      }
    }

    return { nodes, edges };
  },
};
