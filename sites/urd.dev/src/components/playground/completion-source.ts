import type { CompletionContext, CompletionResult, Completion } from '@codemirror/autocomplete';
import { getState, type PlaygroundState } from './playground-state';

/**
 * CodeMirror completion source for Urd Schema Markdown.
 * Three trigger contexts:
 *   @          → entity completions
 *   @entity.   → property completions for that entity's type
 *   ->         → section name completions
 */
export function urdCompletionSource(ctx: CompletionContext): CompletionResult | null {
  const state = getState();
  const line = ctx.state.doc.lineAt(ctx.pos);
  const before = line.text.slice(0, ctx.pos - line.from);

  // 1. Property completion: @entity_id.
  const propMatch = before.match(/@([a-zA-Z_]\w*)\.(\w*)$/);
  if (propMatch) {
    const entityId = propMatch[1];
    const options = completeEntityProperties(state, entityId);
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
      options: completeEntities(state),
    };
  }

  // 3. Section completion: ->
  const sectionMatch = before.match(/->\s*(\w*)$/);
  if (sectionMatch) {
    return {
      from: ctx.pos - sectionMatch[1].length,
      options: completeSections(state),
    };
  }

  // 4. Enum value completion after == != or = on condition/effect lines
  const enumMatch = before.match(/@(\w+)\.(\w+)\s*(?:==|!=|=)\s*(\w*)$/);
  if (enumMatch) {
    const entityId = enumMatch[1];
    const property = enumMatch[2];
    const partial = enumMatch[3];
    const options = completeEnumValues(state, entityId, property);
    if (options.length > 0) {
      return { from: ctx.pos - partial.length, options };
    }
  }

  // 5. Property override completion inside {}
  const overrideResult = completePropertyOverride(ctx, before, state);
  if (overrideResult) return overrideResult;

  return null;
}

function completeEntities(state: PlaygroundState): Completion[] {
  const entities = state.parsedWorld?.entities;
  if (!entities) return [];
  return Object.entries(entities).map(([id, entity]: [string, any]) => ({
    label: id,
    type: 'variable',
    detail: entity.type ?? '',
  }));
}

function completeEntityProperties(state: PlaygroundState, entityId: string): Completion[] {
  const entity = state.parsedWorld?.entities?.[entityId];
  if (!entity) return [];
  const typeName = entity.type;
  const properties = state.parsedWorld?.types?.[typeName]?.properties;
  if (!properties) return [];
  return Object.entries(properties).map(([name, prop]: [string, any]) => ({
    label: name,
    type: 'property',
    detail: prop.type ?? '',
  }));
}

function completeSections(state: PlaygroundState): Completion[] {
  if (!state.definitionIndex) return [];
  const seen = new Set<string>();
  return state.definitionIndex
    .filter((d) => d.definition.kind === 'section')
    .filter((d) => {
      const name = d.definition.local_name!;
      if (seen.has(name)) return false;
      seen.add(name);
      return true;
    })
    .map((d) => ({
      label: d.definition.local_name!,
      type: 'text',
      detail: `in ${d.definition.file_stem}`,
    }));
}

function completeEnumValues(state: PlaygroundState, entityId: string, property: string): Completion[] {
  const entity = state.parsedWorld?.entities?.[entityId];
  if (!entity) return [];
  const typeName = entity.type;
  const prop = state.parsedWorld?.types?.[typeName]?.properties?.[property];
  if (!prop) return [];
  // Only trigger for enum properties
  if (prop.type !== 'enum' && !prop.values) return [];
  const values: string[] = prop.values ?? [];
  return values.map((v: string) => ({
    label: v,
    type: 'enum',
    detail: `${typeName}.${property}`,
  }));
}

function completePropertyOverride(
  ctx: CompletionContext,
  before: string,
  state: PlaygroundState,
): CompletionResult | null {
  // Detect cursor inside {} on an entity declaration line
  // Pattern: @id: TypeName { ... | ... }
  const lineText = ctx.state.doc.lineAt(ctx.pos).text;
  const braceOpen = lineText.indexOf('{');
  const braceClose = lineText.indexOf('}');
  const colInLine = ctx.pos - ctx.state.doc.lineAt(ctx.pos).from;
  if (braceOpen < 0 || colInLine <= braceOpen) return null;
  if (braceClose >= 0 && colInLine > braceClose) return null;

  // Extract TypeName from before the brace
  const preBrace = lineText.slice(0, braceOpen).trim();
  const declMatch = preBrace.match(/@\w+\s*:\s*(\w+)/);
  if (!declMatch) return null;
  const typeName = declMatch[1];

  const typeProps = state.parsedWorld?.types?.[typeName]?.properties;
  if (!typeProps) return null;

  // Determine if we're at a key position or value position
  const insideBraces = lineText.slice(braceOpen + 1, braceClose >= 0 ? braceClose : undefined);
  const cursorInBraces = before.slice(before.indexOf('{') + 1);

  // Value position: after `property_name:` or `property_name =`
  const valueMatch = cursorInBraces.match(/(\w+)\s*[:=]\s*(\w*)$/);
  if (valueMatch) {
    const propName = valueMatch[1];
    const partial = valueMatch[2];
    const prop = typeProps[propName];
    if (!prop) return null;
    const options = completePropertyValue(state, typeName, propName, prop);
    if (options.length > 0) {
      return { from: ctx.pos - partial.length, options };
    }
    return null;
  }

  // Key position: start of token or after comma
  const keyMatch = cursorInBraces.match(/(?:^|,)\s*(\w*)$/);
  if (!keyMatch) return null;
  const partial = keyMatch[1];

  // Exclude properties already present in the override block
  const existing = new Set(
    [...insideBraces.matchAll(/(\w+)\s*[:=]/g)].map((m) => m[1]),
  );

  const options: Completion[] = Object.entries(typeProps)
    .filter(([name]) => !existing.has(name))
    .map(([name, prop]: [string, any]) => {
      const defaultVal = prop.default != null ? ` = ${prop.default}` : '';
      return {
        label: name,
        type: 'property',
        detail: `${prop.type ?? ''}${defaultVal}`,
      };
    });

  if (options.length === 0) return null;
  return { from: ctx.pos - partial.length, options };
}

function completePropertyValue(
  state: PlaygroundState,
  typeName: string,
  propName: string,
  prop: any,
): Completion[] {
  const propType: string = prop.type ?? '';

  // Enum → value list
  if (propType === 'enum' || prop.values) {
    return (prop.values ?? []).map((v: string) => ({
      label: v,
      type: 'enum',
      detail: `${typeName}.${propName}`,
    }));
  }

  // Boolean → true/false
  if (propType === 'bool') {
    return [
      { label: 'true', type: 'keyword' },
      { label: 'false', type: 'keyword' },
    ];
  }

  // Ref → entity IDs of target type
  if (propType.startsWith('ref(')) {
    const refMatch = propType.match(/^ref\((\w+)\)$/);
    if (refMatch && state.parsedWorld?.entities) {
      const targetType = refMatch[1];
      return Object.entries(state.parsedWorld.entities)
        .filter(([, entity]: [string, any]) => entity.type === targetType)
        .map(([id]) => ({
          label: id,
          type: 'variable',
          detail: targetType,
        }));
    }
  }

  return [];
}
