/**
 * Autocomplete extension for the Forge editor.
 *
 * Adapted from the playground's completion-source.ts. Instead of getState(),
 * accepts a getWorldData callback provided by CodeEditorZone.
 *
 * Five trigger contexts:
 *   @            -> entity completions
 *   @entity.     -> property completions for that entity's type
 *   ->           -> section name completions
 *   == != =      -> enum value completions
 *   {}           -> property override completions
 */

import type { CompletionContext, CompletionResult, Completion } from '@codemirror/autocomplete';
import type { DefinitionEntry } from './urd-navigation';

// --- Types ---

export interface AutocompleteDataProvider {
  /** Get world data for entity/type/property lists. */
  getWorldData(): WorldData | null;
  /** Get the definition index for section names. */
  getDefinitionIndex(): DefinitionEntry[] | null;
}

export interface WorldData {
  entities?: Record<string, EntityData> | EntityData[];
  types?: Record<string, TypeData>;
  locations?: LocationData[];
}

export interface EntityData {
  id?: string;
  name?: string;
  type?: string;
  properties?: Record<string, unknown>;
}

export interface TypeData {
  properties?: Record<string, PropertyData>;
}

export interface PropertyData {
  type?: string;
  default?: unknown;
  values?: string[];
}

export interface LocationData {
  id: string;
  name: string;
}

// --- Completion source ---

/**
 * Creates the Urd autocomplete source.
 */
export function createUrdCompletionSource(getProvider: () => AutocompleteDataProvider | null) {
  return function urdCompletionSource(ctx: CompletionContext): CompletionResult | null {
    const provider = getProvider();
    const line = ctx.state.doc.lineAt(ctx.pos);
    const before = line.text.slice(0, ctx.pos - line.from);

    // 1. Property completion: @entity_id.
    const propMatch = before.match(/@([a-zA-Z_]\w*)\.(\w*)$/);
    if (propMatch) {
      const entityId = propMatch[1];
      const options = completeEntityProperties(provider, entityId);
      if (options.length > 0) {
        return {
          from: ctx.pos - propMatch[2].length,
          options,
        };
      }
    }

    // 2. Entity completion: @
    const entityMatch = before.match(/@(\w*)$/);
    if (entityMatch) {
      return {
        from: ctx.pos - entityMatch[1].length,
        options: completeEntities(provider),
      };
    }

    // 3. Section completion: ->
    const sectionMatch = before.match(/->\s*(\w*)$/);
    if (sectionMatch) {
      return {
        from: ctx.pos - sectionMatch[1].length,
        options: completeSections(provider),
      };
    }

    // 4. Enum value completion after == != or = on condition/effect lines
    const enumMatch = before.match(/@(\w+)\.(\w+)\s*(?:==|!=|=)\s*(\w*)$/);
    if (enumMatch) {
      const entityId = enumMatch[1];
      const property = enumMatch[2];
      const partial = enumMatch[3];
      const options = completeEnumValues(provider, entityId, property);
      if (options.length > 0) {
        return { from: ctx.pos - partial.length, options };
      }
    }

    // 5. Property override completion inside {}
    const overrideResult = completePropertyOverride(ctx, before, provider);
    if (overrideResult) return overrideResult;

    return null;
  };
}

// --- Completion helpers ---

function getEntitiesMap(provider: AutocompleteDataProvider | null): Record<string, EntityData> {
  const world = provider?.getWorldData();
  if (!world?.entities) return {};

  // Handle both array and record formats
  if (Array.isArray(world.entities)) {
    const map: Record<string, EntityData> = {};
    for (const e of world.entities) {
      const key = e.name ?? e.id ?? '';
      if (key) map[key] = e;
    }
    return map;
  }

  return world.entities;
}

function completeEntities(provider: AutocompleteDataProvider | null): Completion[] {
  const entities = getEntitiesMap(provider);
  return Object.entries(entities).map(([id, entity]) => ({
    label: id,
    type: 'variable' as const,
    detail: entity.type ?? '',
  }));
}

function completeEntityProperties(provider: AutocompleteDataProvider | null, entityId: string): Completion[] {
  const entities = getEntitiesMap(provider);
  const entity = entities[entityId];
  if (!entity) return [];
  const typeName = entity.type;
  if (!typeName) return [];

  const world = provider?.getWorldData();
  const properties = world?.types?.[typeName]?.properties;
  if (!properties) return [];

  return Object.entries(properties).map(([name, prop]) => ({
    label: name,
    type: 'property' as const,
    detail: prop.type ?? '',
  }));
}

function completeSections(provider: AutocompleteDataProvider | null): Completion[] {
  const index = provider?.getDefinitionIndex();
  if (!index) return [];

  const seen = new Set<string>();
  return index
    .filter((d: DefinitionEntry) => d.definition.kind === 'section')
    .filter((d: DefinitionEntry) => {
      const name = d.definition.local_name ?? d.definition.display_name ?? '';
      if (seen.has(name)) return false;
      seen.add(name);
      return true;
    })
    .map((d: DefinitionEntry) => ({
      label: d.definition.local_name ?? d.definition.display_name ?? '',
      type: 'text' as const,
      detail: d.span.file,
    }));
}

function completeEnumValues(provider: AutocompleteDataProvider | null, entityId: string, property: string): Completion[] {
  const entities = getEntitiesMap(provider);
  const entity = entities[entityId];
  if (!entity) return [];
  const typeName = entity.type;
  if (!typeName) return [];

  const world = provider?.getWorldData();
  const prop = world?.types?.[typeName]?.properties?.[property];
  if (!prop) return [];

  if (prop.type !== 'enum' && !prop.values) return [];
  const values: string[] = prop.values ?? [];
  return values.map((v: string) => ({
    label: v,
    type: 'enum' as const,
    detail: `${typeName}.${property}`,
  }));
}

function completePropertyOverride(
  ctx: CompletionContext,
  before: string,
  provider: AutocompleteDataProvider | null,
): CompletionResult | null {
  const lineText = ctx.state.doc.lineAt(ctx.pos).text;
  const braceOpen = lineText.indexOf('{');
  const braceClose = lineText.indexOf('}');
  const colInLine = ctx.pos - ctx.state.doc.lineAt(ctx.pos).from;
  if (braceOpen < 0 || colInLine <= braceOpen) return null;
  if (braceClose >= 0 && colInLine > braceClose) return null;

  const preBrace = lineText.slice(0, braceOpen).trim();
  const declMatch = preBrace.match(/@\w+\s*:\s*(\w+)/);
  if (!declMatch) return null;
  const typeName = declMatch[1];

  const world = provider?.getWorldData();
  const typeProps = world?.types?.[typeName]?.properties;
  if (!typeProps) return null;

  const insideBraces = lineText.slice(braceOpen + 1, braceClose >= 0 ? braceClose : undefined);
  const cursorInBraces = before.slice(before.indexOf('{') + 1);

  // Value position
  const valueMatch = cursorInBraces.match(/(\w+)\s*[:=]\s*(\w*)$/);
  if (valueMatch) {
    const propName = valueMatch[1];
    const partial = valueMatch[2];
    const prop = typeProps[propName];
    if (!prop) return null;
    const options = completePropertyValue(provider, typeName, propName, prop);
    if (options.length > 0) {
      return { from: ctx.pos - partial.length, options };
    }
    return null;
  }

  // Key position
  const keyMatch = cursorInBraces.match(/(?:^|,)\s*(\w*)$/);
  if (!keyMatch) return null;
  const partial = keyMatch[1];

  const existing = new Set(
    [...insideBraces.matchAll(/(\w+)\s*[:=]/g)].map((m) => m[1]),
  );

  const options: Completion[] = Object.entries(typeProps)
    .filter(([name]) => !existing.has(name))
    .map(([name, prop]) => {
      const defaultVal = prop.default != null ? ` = ${String(prop.default)}` : '';
      return {
        label: name,
        type: 'property' as const,
        detail: `${prop.type ?? ''}${defaultVal}`,
      };
    });

  if (options.length === 0) return null;
  return { from: ctx.pos - partial.length, options };
}

function completePropertyValue(
  provider: AutocompleteDataProvider | null,
  typeName: string,
  propName: string,
  prop: PropertyData,
): Completion[] {
  const propType: string = prop.type ?? '';

  // Enum values
  if (propType === 'enum' || prop.values) {
    return (prop.values ?? []).map((v: string) => ({
      label: v,
      type: 'enum' as const,
      detail: `${typeName}.${propName}`,
    }));
  }

  // Boolean
  if (propType === 'bool') {
    return [
      { label: 'true', type: 'keyword' as const },
      { label: 'false', type: 'keyword' as const },
    ];
  }

  // Ref — entity IDs of target type
  if (propType.startsWith('ref(')) {
    const refMatch = propType.match(/^ref\((\w+)\)$/);
    if (refMatch) {
      const targetType = refMatch[1];
      const entities = getEntitiesMap(provider);
      return Object.entries(entities)
        .filter(([, entity]) => entity.type === targetType)
        .map(([id]) => ({
          label: id,
          type: 'variable' as const,
          detail: targetType,
        }));
    }
  }

  return [];
}
