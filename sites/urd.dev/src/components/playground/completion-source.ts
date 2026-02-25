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
