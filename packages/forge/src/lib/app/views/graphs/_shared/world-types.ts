/**
 * World canvas data model — rich spatial representation of an Urd world.
 * Used by WorldCanvas.svelte to render locations, entities, exits, scenes,
 * and choices on a giant pannable/zoomable canvas.
 */

// ===== Layout nodes =====

export interface WorldLocation {
  id: string;
  name: string;
  description: string;
  x: number;
  y: number;
  width: number;
  height: number;
  isStart: boolean;
  entities: WorldEntity[];
  exits: WorldExit[];
}

export interface WorldEntity {
  id: string;
  name: string;
  type?: string;
  /** Resolved entity kind for icon rendering. */
  entityKind: EntityIconKind;
  properties: Record<string, unknown>;
}

export type EntityIconKind =
  | 'character'
  | 'item'
  | 'lock'
  | 'container'
  | 'scenery'
  | 'property'
  | 'unknown';

export interface WorldExit {
  direction: string;
  targetId: string;
  isConditional: boolean;
  /** Human-readable condition text (e.g. "garden_gate.locked = false"). */
  conditionLabel?: string;
}

export interface WorldScene {
  id: string;
  locationId?: string;
  choices: WorldChoice[];
  x: number;
  y: number;
}

export interface WorldChoice {
  id: string;
  label: string;
  sticky: boolean;
  /** Number of property mutations this choice performs. */
  effectCount: number;
}

// ===== Runtime overlay =====

export interface WorldRuntimeOverlay {
  currentLocationId: string | null;
  visitedLocationIds: Set<string>;
  visitedExitKeys: Set<string>;
  entityStates: Map<string, Record<string, unknown>>;
  turnCount: number;
  isPlaying: boolean;
}

// ===== Full world data =====

export interface WorldData {
  locations: WorldLocation[];
  scenes: WorldScene[];
  worldName: string;
}

// ===== Entity kind resolution =====

/** Resolve an entity's type name to an icon kind. */
export function resolveEntityKind(typeName?: string, traits?: string[]): EntityIconKind {
  if (!typeName) return 'unknown';
  const lower = typeName.toLowerCase();

  // Check well-known type names
  if (lower.includes('character') || lower.includes('npc') || lower.includes('person') || lower.includes('player')) return 'character';
  if (lower.includes('lock') || lower.includes('door') || lower.includes('gate') || lower.includes('barrier')) return 'lock';
  if (lower.includes('container') || lower.includes('chest') || lower.includes('box') || lower.includes('bag')) return 'container';
  if (lower.includes('item') || lower.includes('key') || lower.includes('weapon') || lower.includes('tool') || lower.includes('object')) return 'item';
  if (lower.includes('scenery') || lower.includes('decoration') || lower.includes('prop')) return 'scenery';

  // Check traits
  if (traits) {
    for (const trait of traits) {
      const t = trait.toLowerCase();
      if (t.includes('character') || t.includes('npc')) return 'character';
      if (t.includes('lock') || t.includes('openable')) return 'lock';
      if (t.includes('container') || t.includes('portable')) return 'container';
      if (t.includes('item') || t.includes('takeable')) return 'item';
    }
  }

  return 'unknown';
}
