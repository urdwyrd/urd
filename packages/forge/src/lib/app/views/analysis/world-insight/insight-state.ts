/**
 * Insight state — gathers all projection data into a single typed object.
 *
 * Called once per `compiler.completed` signal. Section components receive
 * slices as props, never calling the projection registry directly.
 */

import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
import type {
  UrdWorld,
  FactSet,
  PropertyDependencyIndex,
  SymbolTable,
  SymbolTableEntry,
} from '$lib/app/compiler/types';
import type { WorldStats } from '$lib/app/projections/world-stats';
import type { EntityRow } from '$lib/app/projections/entity-table';
import type { FileDiagnostics } from '$lib/app/projections/diagnostics-by-file';

export interface InsightState {
  world: UrdWorld | null;
  factSet: FactSet | null;
  propertyIndex: PropertyDependencyIndex | null;
  diagnosticsByFile: FileDiagnostics[] | null;
  entityTable: EntityRow[] | null;
  symbolTable: SymbolTable | null;
  worldStats: WorldStats | null;
  /** Reverse map: entity ID → containing location ID. */
  entityLocationMap: Map<string, string>;
  /** Symbol lookup by ID or name. */
  symbolMap: Map<string, SymbolTableEntry>;
}

export function buildInsightState(): InsightState {
  const world = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
  const symbolTable = projectionRegistry.get<SymbolTable>('urd.projection.symbolTable');

  // Build entity → location reverse map
  const entityLocationMap = new Map<string, string>();
  if (world) {
    for (const loc of world.locations) {
      if (loc.contains) {
        for (const entityId of loc.contains) {
          entityLocationMap.set(entityId, loc.id);
        }
      }
    }
  }

  // Build symbol lookup map
  const symbolMap = new Map<string, SymbolTableEntry>();
  if (symbolTable) {
    for (const entry of symbolTable.entries) {
      symbolMap.set(entry.id, entry);
      if (entry.name && !symbolMap.has(entry.name)) {
        symbolMap.set(entry.name, entry);
      }
    }
  }

  return {
    world,
    factSet: projectionRegistry.get<FactSet>('urd.projection.factSet'),
    propertyIndex: projectionRegistry.get<PropertyDependencyIndex>('urd.projection.propertyDependencyIndex'),
    diagnosticsByFile: projectionRegistry.get<FileDiagnostics[]>('urd.projection.diagnosticsByFile'),
    entityTable: projectionRegistry.get<EntityRow[]>('urd.projection.entityTable'),
    symbolTable,
    worldStats: projectionRegistry.get<WorldStats>('urd.projection.worldStats'),
    entityLocationMap,
    symbolMap,
  };
}
