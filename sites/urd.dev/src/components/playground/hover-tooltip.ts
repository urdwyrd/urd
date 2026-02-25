import { hoverTooltip } from '@codemirror/view';
import { identifyReference, type Reference } from './cursor-resolver';
import { getState, type PlaygroundState } from './playground-state';

/**
 * CodeMirror hover tooltip extension showing rich FactSet data
 * for entities, properties, sections, and locations.
 */
export function urdHoverTooltip() {
  return hoverTooltip((view, pos) => {
    const line = view.state.doc.lineAt(pos);
    const col = pos - line.from;
    const ref = identifyReference(line.text, col);
    if (!ref) return null;

    const state = getState();
    const content = buildTooltipContent(ref, state);
    if (!content) return null;

    return {
      pos,
      above: true,
      create: () => {
        const dom = document.createElement('div');
        dom.className = 'urd-hover-tooltip';
        dom.innerHTML = content;
        return { dom };
      },
    };
  });
}

function buildTooltipContent(ref: Reference, state: PlaygroundState): string | null {
  switch (ref.kind) {
    case 'entity':
      return entityTooltip(ref.id, state);
    case 'entity-property':
      return propertyTooltip(ref.entityId, ref.property, state);
    case 'type-property':
      return typePropertyTooltip(ref.typeName, ref.property, state);
    case 'section-jump':
    case 'section-label':
      return sectionTooltip(ref.name, state);
    case 'location-heading':
      return locationTooltip(ref.name, state);
    default:
      return null;
  }
}

function entityTooltip(id: string, state: PlaygroundState): string | null {
  const entity = state.parsedWorld?.entities?.[id];
  if (!entity) return null;

  const typeName = entity.type ?? 'unknown';
  let html = `<strong>@${esc(id)}</strong>: ${esc(typeName)}`;

  // Find container location
  const locations = state.parsedWorld?.locations;
  if (locations) {
    for (const [locId, loc] of Object.entries(locations) as [string, any][]) {
      if (loc.contains?.includes(id)) {
        html += `<br><span class="urd-tt-dim">Container: ${esc(locId)}</span>`;
        break;
      }
    }
  }

  // List properties
  const props = entity.properties;
  if (props && Object.keys(props).length > 0) {
    const pairs = Object.entries(props)
      .map(([k, v]) => `${esc(k)}: ${esc(String(v))}`)
      .join(', ');
    html += `<br><span class="urd-tt-dim">Properties: ${pairs}</span>`;
  }

  return html;
}

function propertyTooltip(entityId: string, property: string, state: PlaygroundState): string | null {
  const typeName = resolveEntityType(entityId, state);
  if (!typeName) return null;
  return typePropertyTooltip(typeName, property, state);
}

function typePropertyTooltip(typeName: string, property: string, state: PlaygroundState): string | null {
  // Look up in DefinitionIndex for type and default
  const key = `prop:${typeName}.${property}`;
  const entry = state.definitionIndex?.find((d) => d.key === key);

  let propType = entry?.definition.property_type ?? 'unknown';
  let defaultVal = entry?.definition.default;
  let html = `<strong>${esc(typeName)}.${esc(property)}</strong>: ${esc(propType)}`;

  if (defaultVal != null) {
    html += ` <span class="urd-tt-dim">(default: ${esc(defaultVal)})</span>`;
  }

  // Read/write counts from PropertyDependencyIndex
  const propIndex = state.result?.property_index;
  if (propIndex) {
    const propEntry = propIndex.properties.find(
      (p) => p.entity_type === typeName && p.property === property,
    );
    if (propEntry) {
      html += `<br><span class="urd-tt-dim">Read by: ${propEntry.read_count} sites · Written by: ${propEntry.write_count} sites</span>`;
      if (propEntry.read_count > 0 && propEntry.write_count === 0) {
        html += `<br><span class="urd-tt-warn">Read but never written</span>`;
      } else if (propEntry.write_count > 0 && propEntry.read_count === 0) {
        html += `<br><span class="urd-tt-warn">Written but never read</span>`;
      }
    }
  }

  return html;
}

function sectionTooltip(name: string, state: PlaygroundState): string | null {
  if (!state.definitionIndex) return null;

  // Find section in DefinitionIndex
  const entry = state.definitionIndex.find(
    (d) => d.definition.kind === 'section' && d.definition.local_name === name,
  );
  if (!entry) return null;

  const compiledId = entry.key.replace(/^section:/, '');
  let html = `<strong>Section</strong>: ${esc(compiledId)}`;

  // Count jumps and choices from FactSet
  const facts = state.result?.facts;
  if (facts) {
    const incoming = facts.jumps.filter(
      (j: any) => j.target?.kind === 'section' && j.target?.id === compiledId,
    ).length;
    const outgoing = facts.jumps.filter(
      (j: any) => j.from_section === compiledId,
    ).length;
    const choices = facts.choices.filter(
      (c: any) => c.section === compiledId,
    ).length;

    html += `<br><span class="urd-tt-dim">Incoming: ${incoming} · Outgoing: ${outgoing}</span>`;
    html += `<br><span class="urd-tt-dim">Choices: ${choices}</span>`;
  }

  return html;
}

function locationTooltip(displayName: string, state: PlaygroundState): string | null {
  if (!state.definitionIndex) return null;

  const entry = state.definitionIndex.find(
    (d) => d.definition.kind === 'location' && d.definition.display_name === displayName,
  );
  if (!entry) return null;

  const slug = entry.key.replace(/^location:/, '');
  let html = `<strong>Location</strong>: ${esc(slug)}`;

  const location = state.parsedWorld?.locations?.[slug];
  if (location) {
    const exits = location.exits?.length ?? 0;
    const entities = location.contains?.length ?? 0;
    html += `<br><span class="urd-tt-dim">Exits: ${exits} · Entities: ${entities}</span>`;
  }

  return html;
}

function resolveEntityType(entityId: string, state: PlaygroundState): string | null {
  return state.parsedWorld?.entities?.[entityId]?.type ?? null;
}

function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
