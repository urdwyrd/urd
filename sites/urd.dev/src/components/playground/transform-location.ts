/**
 * Transform FactSet exit data into a location topology graph.
 *
 * Nodes: one per location from compiled world JSON (plus any extras from exits).
 * Edges: one per ExitEdge from the FactSet.
 * Flags: cross-reference URD430 diagnostics for unreachable locations.
 */

import type { FactSet, Diagnostic } from './compiler-bridge';
import type { GraphData, GraphNode, GraphEdge } from './graph-types';

export function buildLocationGraph(
  facts: FactSet,
  worldJson: string | null,
  diagnostics: Diagnostic[],
): GraphData {
  const nodes: GraphNode[] = [];
  const edges: GraphEdge[] = [];

  // Collect location IDs from world JSON (authoritative node set).
  const locationIds = new Set<string>();
  let startLocation: string | null = null;

  if (worldJson) {
    try {
      const world = JSON.parse(worldJson);
      if (world.locations) {
        for (const locId of Object.keys(world.locations)) {
          locationIds.add(locId);
        }
      }
      if (world.world?.start) {
        startLocation = world.world.start;
      }
    } catch { /* ignore parse failures */ }
  }

  // Also collect location IDs from exits (handles partial compilation).
  for (const exit of facts.exits) {
    locationIds.add(exit.from_location);
    locationIds.add(exit.to_location);
  }

  // Parse URD430 diagnostics — extract unreachable location slugs.
  const unreachableLocations = new Set<string>();
  for (const diag of diagnostics) {
    if (diag.code === 'URD430') {
      const match = diag.message.match(/Location '([^']+)' is unreachable/);
      if (match) unreachableLocations.add(match[1]);
    }
  }

  // Build location set referenced by exits.
  const exitReferenced = new Set<string>();
  for (const exit of facts.exits) {
    exitReferenced.add(exit.from_location);
    exitReferenced.add(exit.to_location);
  }

  // Build nodes.
  for (const locId of [...locationIds].sort()) {
    const isIsolated = !exitReferenced.has(locId);
    nodes.push({
      id: locId,
      label: locId.replace(/-/g, ' '),
      kind: 'location',
      flag: unreachableLocations.has(locId) ? 'unreachable' : null,
    });
  }

  // Build edges from ExitEdge tuples.
  for (const exit of facts.exits) {
    edges.push({
      from: exit.from_location,
      to: exit.to_location,
      label: exit.exit_name,
      conditional: exit.is_conditional,
    });
  }

  return { nodes, edges };
}
