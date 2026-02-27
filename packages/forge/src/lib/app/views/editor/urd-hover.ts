/**
 * Hover tooltip extension for the Forge editor.
 *
 * Direct port of the playground's hover-tooltip.ts. Uses the raw
 * object-keyed parsedWorld (not normalised arrays) so that all
 * lookup patterns are identical to the playground.
 */

import { hoverTooltip } from '@codemirror/view';
import { identifyReference, getFrontmatterContext, type Reference } from './cursor-resolver';
import type { DefinitionEntry } from './urd-navigation';
import type { FactSet, PropertyDependencyIndex, Diagnostic } from '$lib/app/compiler/types';

// --- Types ---

export interface HoverDataProvider {
  /** Raw parsedWorld — object-keyed, exactly like the playground's state.parsedWorld. */
  getParsedWorld(): Record<string, unknown> | null;
  /** Get the definition index entries. */
  getDefinitionIndex(): DefinitionEntry[] | null;
  /** Get the full FactSet from the compiler. */
  getFactSet(): FactSet | null;
  /** Get the PropertyDependencyIndex from the compiler. */
  getPropertyDependencyIndex(): PropertyDependencyIndex | null;
  /** Get diagnostics for the current file. */
  getDiagnostics?(): Diagnostic[] | null;
  /** Get the entity table for reference counting. */
  getEntityTable?(): { id: string; name: string }[] | null;
  /** Get the property table for reference counting. */
  getPropertyTable?(): { object: string }[] | null;
}

// --- Hover tooltip extension ---

export function urdHoverTooltip(getProvider: () => HoverDataProvider | null) {
  return hoverTooltip((view, pos) => {
    const line = view.state.doc.lineAt(pos);
    const col = pos - line.from;
    if (import.meta.env?.DEV && col === 0) {
      console.warn('[urd-hover] handler fired at line', line.number, 'col', col);
    }
    const context = getFrontmatterContext(view.state.doc, line.number);
    const ref = identifyReference(line.text, col, context);
    if (!ref) return null;

    const provider = getProvider();

    // Trace hover data flow (always log in dev, stripped in prod)
    if (import.meta.env?.DEV) {
      const pw = provider?.getParsedWorld();
      const di = provider?.getDefinitionIndex();
      const fs = provider?.getFactSet();
      const pi = provider?.getPropertyDependencyIndex();
      // Spread arrays so the console prints actual values, not "Array(N)"
      console.warn('[urd-hover] ref:', ref.kind, JSON.stringify(ref), {
        entityKeys: pw?.entities ? [...Object.keys(pw.entities as Record<string, unknown>)] : 'no-pw',
        defKeys: di ? [...di.map((d) => d.key)] : 'no-di',
        defSample: di && di.length > 0 ? JSON.stringify(di[0]) : 'empty',
        factReads: fs?.reads?.length ?? 0,
        propIdx: pi?.properties?.length ?? 0,
      });
    }

    let content = buildTooltipContent(ref, provider, view, line.number);
    if (!content) {
      if (import.meta.env?.DEV) {
        console.warn('[urd-hover] buildTooltipContent returned null for', ref.kind);
      }
      return null;
    }

    // Prepend relevant diagnostics for this line (matching playground)
    const diagnostics = provider?.getDiagnostics?.();
    if (diagnostics) {
      const lineDiags = diagnostics.filter(
        (d) => d.span && d.span.startLine <= line.number && d.span.endLine >= line.number,
      );
      if (lineDiags.length > 0) {
        const diagHtml = lineDiags.map((d) => {
          const cls = d.severity === 'error' ? 'urd-tt-error' : 'urd-tt-warn';
          return `<span class="${cls}">[${esc(d.code)}] ${esc(d.message)}</span>`;
        }).join('<br>');
        content = diagHtml + '<br>' + content;
      }
    }

    // Presence marker validation: [@entity] shows containment status
    if (ref.kind === 'entity') {
      content = appendPresenceValidation(content, ref.id, line.text, col, view, line.number, provider);
      content = appendEntityContextAnnotation(content, ref.id, line.text, col);
    }

    // Default value hints on condition/effect lines
    if (ref.kind === 'entity-property') {
      content = appendDefaultValueHint(content, ref.entityId, ref.property, line.text, provider);
    }

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

// --- Tooltip builders ---

function buildTooltipContent(
  ref: Reference,
  provider: HoverDataProvider | null,
  view?: unknown,
  lineNumber?: number,
): string | null {
  switch (ref.kind) {
    case 'entity':
      return entityTooltip(ref.id, provider);
    case 'entity-property':
      return propertyTooltip(ref.entityId, ref.property, provider);
    case 'type-property':
      return typePropertyTooltip(ref.typeName, ref.property, provider);
    case 'section-jump':
    case 'section-label':
      return sectionTooltip(ref.name, provider, view);
    case 'location-heading':
      return locationTooltip(ref.name, provider);
    case 'keyword':
      return keywordTooltip(ref.token);
    case 'frontmatter-key':
      return frontmatterKeyTooltip(ref.key);
    case 'type-constructor':
      return typeConstructorTooltip(ref.name, ref.range, ref.defaultValue, ref.enumValues);
    case 'exit-direction':
      return exitDirectionTooltip(ref.direction, view, lineNumber ?? 0, provider);
    case 'exit-destination':
      return exitDestinationTooltip(ref.destinationName, provider);
    case 'trait':
      return traitTooltip(ref.name);
    case 'visibility-prefix':
      return visibilityTooltip();
    case 'effect-command':
      return effectCommandTooltip(ref.command);
    case 'condition-keyword':
      return conditionKeywordTooltip(ref.keyword);
    case 'condition-combinator':
      return conditionCombinatorTooltip(ref.combinator);
    case 'rule-keyword':
      return ruleKeywordTooltip(ref.keyword);
    case 'rule-name':
      return ruleNameTooltip(ref.name, provider);
    case 'sequence-heading':
      return sequenceHeadingTooltip(ref.name, provider);
    case 'phase-heading':
      return phaseHeadingTooltip(ref.name, ref.auto);
    case 'type-name':
      return typeNameTooltip(ref.name, ref.traits, provider);
    case 'world-sub-key':
      return worldSubKeyTooltip(ref.key, ref.value, provider);
    case 'exit-jump':
      return exitJumpTooltip(ref.direction, view, lineNumber ?? 0, provider);
    case 'value-literal':
      return valueLiteralTooltip(ref.value, ref.entityId, ref.property, provider);
    case 'comment':
      return commentTooltip();
    default:
      return null;
  }
}

// ─── Entity tooltip ────────────────────────────────────────────────
// Playground: state.parsedWorld?.entities?.[id]  (object-keyed)

function entityTooltip(id: string, provider: HoverDataProvider | null): string | null {
  const pw = provider?.getParsedWorld();
  const entity = (pw?.entities as Record<string, Record<string, unknown>> | undefined)?.[id];

  // Fall back to definitionIndex when parsedWorld has no data
  if (!entity) {
    const defIndex = provider?.getDefinitionIndex();
    const entry = defIndex?.find((d) => d.key === `entity:@${id}`);
    if (!entry) return null;
    const typeName = entry.definition.type_name ?? 'unknown';
    return `<strong>@${esc(id)}</strong>: ${esc(typeName)}`;
  }

  const typeName = (entity.type as string) ?? 'unknown';
  let html = `<strong>@${esc(id)}</strong>: ${esc(typeName)}`;

  // Find container location
  const locations = pw?.locations as Record<string, Record<string, unknown>> | undefined;
  if (locations) {
    for (const [locId, loc] of Object.entries(locations)) {
      if ((loc.contains as string[] | undefined)?.includes(id)) {
        html += `<br><span class="urd-tt-dim">Container: ${esc(locId)}</span>`;
        break;
      }
    }
  }

  // List properties
  const props = entity.properties as Record<string, unknown> | undefined;
  if (props && Object.keys(props).length > 0) {
    const pairs = Object.entries(props)
      .map(([k, v]) => `${esc(k)}: ${esc(String(v))}`)
      .join(', ');
    html += `<br><span class="urd-tt-dim">Properties: ${pairs}</span>`;
  }

  // Reference count
  const refCount = countEntityReferences(id, provider);
  if (refCount > 0) {
    html += `<br><span class="urd-tt-dim">${refCount} reference${refCount === 1 ? '' : 's'}</span>`;
  }

  return html;
}

// ─── Property tooltip (instance-level: @entity.property) ───────────

function propertyTooltip(entityId: string, property: string, provider: HoverDataProvider | null): string | null {
  const typeName = resolveEntityType(entityId, provider);
  if (!typeName) return null;
  return typePropertyTooltip(typeName, property, provider);
}

// ─── Type property tooltip (schema-level: TypeName.property) ───────
// Playground: definitionIndex for type/default, property_index for read/write

function typePropertyTooltip(typeName: string, property: string, provider: HoverDataProvider | null): string | null {
  const defIndex = provider?.getDefinitionIndex();

  // Implicit container property
  if (property === 'container') {
    const key = `prop:${typeName}.container`;
    const entry = defIndex?.find((d) => d.key === key);
    if (!entry) {
      let html = `<strong>${esc(typeName)}.container</strong> <span class="urd-tt-dim">(implicit)</span>`;
      html += `<br>The entity or location that currently holds this entity.`;
      html += `<br><span class="urd-tt-dim">Value is an entity ID, location slug, or "player".</span>`;
      html += `<br><span class="urd-tt-dim">Read-only — changed via move effects, not direct assignment.</span>`;
      return html;
    }
  }

  // Look up in DefinitionIndex for type and default
  const key = `prop:${typeName}.${property}`;
  const entry = defIndex?.find((d) => d.key === key);

  const propType = entry?.definition.property_type ?? 'unknown';
  const defaultVal = entry?.definition.default;

  if (import.meta.env?.DEV) {
    console.warn('[urd-hover] typePropertyTooltip', { key, found: !!entry, propType, defaultVal, entry: entry ? JSON.stringify(entry) : null });
  }

  let html = `<strong>${esc(typeName)}.${esc(property)}</strong>: ${esc(propType)}`;

  if (defaultVal != null) {
    html += ` <span class="urd-tt-dim">(default: ${esc(String(defaultVal))})</span>`;
  }

  // Read/write counts from PropertyDependencyIndex
  const propIndex = provider?.getPropertyDependencyIndex();
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

// ─── Section tooltip ───────────────────────────────────────────────

function sectionTooltip(name: string, provider: HoverDataProvider | null, view?: unknown): string | null {
  const defIndex = provider?.getDefinitionIndex();
  if (!defIndex) return null;

  const entry = defIndex.find(
    (d) => d.definition.kind === 'section' && d.definition.local_name === name,
  );
  if (!entry) return null;

  const compiledId = entry.key.replace(/^section:/, '');
  let html = `<strong>Section</strong>: ${esc(compiledId)}`;

  // Count jumps and choices from FactSet
  const facts = provider?.getFactSet();
  if (facts) {
    const incoming = facts.jumps.filter(
      (j) => j.target?.kind === 'section' && j.target?.id === compiledId,
    ).length;
    const outgoing = facts.jumps.filter(
      (j) => j.from_section === compiledId,
    ).length;
    const choices = facts.choices.filter(
      (c) => c.section === compiledId,
    ).length;

    html += `<br><span class="urd-tt-dim">Incoming: ${incoming} · Outgoing: ${outgoing}</span>`;
    html += `<br><span class="urd-tt-dim">Choices: ${choices}</span>`;
  }

  // Content preview — first 5 non-blank lines after the section label
  if (view && entry.span) {
    const preview = getSectionPreview(view, entry.span.start_line);
    if (preview) {
      html += `<br><pre style="margin:4px 0 0;white-space:pre-wrap;color:var(--dim)">${preview}</pre>`;
    }
  }

  return html;
}

function getSectionPreview(view: unknown, sectionStartLine: number): string | null {
  const v = view as { state: { doc: { lines: number; line(n: number): { text: string } } } };
  const doc = v.state.doc;
  const maxLines = doc.lines;
  const collected: string[] = [];
  let hasMore = false;

  for (let i = sectionStartLine + 1; i <= maxLines && collected.length < 5; i++) {
    const text = doc.line(i).text;
    if (/^(==\s|#\s)/.test(text.trimStart())) break;
    if (text.trim().length > 0) {
      collected.push(text);
    }
  }

  if (collected.length === 5) {
    const nextContentLine = sectionStartLine + 1;
    let remaining = 0;
    for (let i = nextContentLine; i <= maxLines; i++) {
      const text = doc.line(i).text;
      if (/^(==\s|#\s)/.test(text.trimStart())) break;
      if (text.trim().length > 0) remaining++;
    }
    if (remaining > 5) hasMore = true;
  }

  if (collected.length === 0) return null;
  let preview = collected.map((l) => esc(l)).join('\n');
  if (hasMore) preview += '\n…';
  return preview;
}

// ─── Location tooltip ──────────────────────────────────────────────

function locationTooltip(displayName: string, provider: HoverDataProvider | null): string | null {
  const defIndex = provider?.getDefinitionIndex();
  if (!defIndex) return `<strong>Location</strong>: ${esc(displayName)}`;

  const entry = defIndex.find(
    (d) => d.definition.kind === 'location' && d.definition.display_name === displayName,
  );
  if (!entry) return `<strong>Location</strong>: ${esc(displayName)}`;

  const slug = entry.key.replace(/^location:/, '');
  let html = `<strong>Location</strong>: ${esc(slug)}`;

  const pw = provider?.getParsedWorld();
  const location = (pw?.locations as Record<string, Record<string, unknown>> | undefined)?.[slug];
  if (location) {
    const exits = location.exits as Record<string, unknown> | unknown[] | undefined;
    const exitCount = exits ? (Array.isArray(exits) ? exits.length : Object.keys(exits).length) : 0;
    const entities = (location.contains as string[] | undefined)?.length ?? 0;
    html += `<br><span class="urd-tt-dim">Exits: ${exitCount} · Entities: ${entities}</span>`;
  }

  return html;
}

// ─── Exit direction tooltip ────────────────────────────────────────
// Playground: state.parsedWorld?.locations?.[enclosing] object-keyed

function exitDirectionTooltip(direction: string, view: unknown, lineNumber: number, provider: HoverDataProvider | null): string | null {
  const enclosing = findEnclosingLocation(view, lineNumber, provider);
  if (!enclosing) return null;

  const pw = provider?.getParsedWorld();
  const location = (pw?.locations as Record<string, Record<string, unknown>> | undefined)?.[enclosing];
  const exits = location?.exits as Record<string, Record<string, unknown>> | undefined;
  if (!exits || typeof exits !== 'object' || Object.keys(exits).length === 0) return null;

  // Get display name from definition index
  const defIndex = provider?.getDefinitionIndex();
  const defEntry = defIndex?.find(
    (d) => d.definition.kind === 'location' && d.key === `location:${enclosing}`,
  );
  const displayName = defEntry?.definition?.display_name ?? enclosing.replace(/-/g, ' ');

  let html = `<strong>Exits from ${esc(displayName)}</strong>`;

  const facts = provider?.getFactSet();
  for (const [dir, exitData] of Object.entries(exits)) {
    const dest = (exitData as Record<string, unknown>).to as string ?? '?';
    let line = `  ${esc(dir)} → ${esc(dest.replace(/-/g, ' '))}`;
    // Check conditional status from FactSet
    if (facts?.exits) {
      const exitFact = facts.exits.find(
        (e) => e.from_location === enclosing && e.exit_name === dir,
      );
      if (exitFact?.is_conditional) {
        line += ' (conditional)';
      }
    }
    const isCurrent = dir === direction;
    html += `<br><span class="urd-tt-dim" style="${isCurrent ? 'font-weight:600' : ''}">${line}</span>`;
  }

  return html;
}

// ─── Exit destination tooltip ──────────────────────────────────────

function exitDestinationTooltip(destinationName: string, provider: HoverDataProvider | null): string | null {
  const pw = provider?.getParsedWorld();
  const locations = pw?.locations as Record<string, Record<string, unknown>> | undefined;
  if (!locations) return null;

  // Look up slug via definition index (display name → slug)
  const defIndex = provider?.getDefinitionIndex();
  let slug: string | null = null;
  const defEntry = defIndex?.find(
    (d) => d.definition.kind === 'location' && d.definition.display_name === destinationName,
  );
  if (defEntry) {
    slug = defEntry.key.replace(/^location:/, '');
  } else {
    const slugified = destinationName.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
    if (locations[slugified]) {
      slug = slugified;
    }
  }
  if (!slug) return null;

  const loc = locations[slug];
  if (!loc) return null;

  const displayName = defEntry?.definition?.display_name ?? destinationName;
  let html = `<strong>Location</strong>: ${esc(displayName)}`;

  if (loc.description) {
    html += `<br><span class="urd-tt-dim">${esc(loc.description as string)}</span>`;
  }

  // Exits (object-keyed: { "east": { "to": "market" } })
  const exits = loc.exits as Record<string, Record<string, unknown>> | undefined;
  if (exits && typeof exits === 'object' && Object.keys(exits).length > 0) {
    const exitList = Object.entries(exits)
      .map(([dir, data]) => `${esc(dir)} → ${esc(((data.to as string) ?? '?').replace(/-/g, ' '))}`)
      .join(', ');
    html += `<br><span class="urd-tt-dim">Exits: ${exitList}</span>`;
  }

  // Contains
  const contains = loc.contains as string[] | undefined;
  if (contains && contains.length > 0) {
    const entityList = contains.map((id: string) => `@${esc(id)}`).join(', ');
    html += `<br><span class="urd-tt-dim">Contains: ${entityList}</span>`;
  }

  return html;
}

// ─── Exit-jump tooltip ─────────────────────────────────────────────

function exitJumpTooltip(direction: string, view: unknown, lineNumber: number, provider: HoverDataProvider | null): string | null {
  let html = `<strong>Exit jump:</strong> ${esc(direction)}`;
  html += `<br><span class="urd-tt-dim">Navigates via the exit named "${esc(direction)}" from the current location.</span>`;

  const enclosing = findEnclosingLocation(view, lineNumber, provider);
  if (enclosing) {
    const pw = provider?.getParsedWorld();
    const location = (pw?.locations as Record<string, Record<string, unknown>> | undefined)?.[enclosing];
    const exits = location?.exits as Record<string, Record<string, unknown>> | undefined;
    if (exits) {
      const exitData = exits[direction];
      if (exitData?.to) {
        html += `<br>Resolves to: <strong>${esc((exitData.to as string).replace(/-/g, ' '))}</strong>`;
      }
    }
  }

  return html;
}

// ─── Presence marker validation ────────────────────────────────────

function appendPresenceValidation(
  html: string,
  entityId: string,
  lineText: string,
  col: number,
  view: unknown,
  lineNumber: number,
  provider: HoverDataProvider | null,
): string {
  const atIdx = lineText.lastIndexOf('@', col);
  if (atIdx < 0) return html;
  if (atIdx < 1 || lineText[atIdx - 1] !== '[') return html;
  const closeBracket = lineText.indexOf(']', atIdx);
  if (closeBracket < 0) return html;

  const enclosing = findEnclosingLocation(view, lineNumber, provider);
  if (!enclosing) return html;

  const pw = provider?.getParsedWorld();
  const location = (pw?.locations as Record<string, Record<string, unknown>> | undefined)?.[enclosing];
  const contained = (location?.contains as string[] | undefined)?.includes(entityId);

  if (contained) {
    html += `<br><span class="urd-tt-dim">✓ Placed in this location (${esc(enclosing)})</span>`;
  } else {
    html += `<br><span class="urd-tt-warn">⚠ Not contained in this location (${esc(enclosing)})</span>`;
    const locations = pw?.locations as Record<string, Record<string, unknown>> | undefined;
    if (locations) {
      for (const [locId, loc] of Object.entries(locations)) {
        if ((loc.contains as string[] | undefined)?.includes(entityId)) {
          html += `<br><span class="urd-tt-warn">  Contained in: ${esc(locId)}</span>`;
          break;
        }
      }
    }
  }
  return html;
}

// ─── Default value hints on condition/effect lines ─────────────────

function appendDefaultValueHint(
  html: string,
  entityId: string,
  property: string,
  lineText: string,
  provider: HoverDataProvider | null,
): string {
  const trimmed = lineText.trimStart();
  const isCondition = trimmed.startsWith('? ');
  const isEffect = trimmed.startsWith('> ');
  if (!isCondition && !isEffect) return html;

  const pw = provider?.getParsedWorld();
  const entity = (pw?.entities as Record<string, Record<string, unknown>> | undefined)?.[entityId];
  if (!entity) return html;
  const typeName = entity.type as string | undefined;
  const typeDef = (pw?.types as Record<string, Record<string, unknown>> | undefined)?.[typeName ?? ''];
  const propDef = (typeDef?.properties as Record<string, Record<string, unknown>> | undefined)?.[property];
  if (!propDef) return html;

  const entityOverride = (entity.properties as Record<string, unknown> | undefined)?.[property];
  const defaultVal = entityOverride ?? propDef.default;
  if (defaultVal == null) return html;

  const defaultLabel = entityOverride != null ? 'override' : 'default';
  html += `<br><span class="urd-tt-dim">@${esc(entityId)}.${esc(property)} = ${esc(String(defaultVal))} (${defaultLabel})</span>`;

  const opMatch = lineText.match(
    new RegExp(`@${entityId}\\.${property}\\s*(>=|<=|==|!=|>|<|\\+|\\-|\\*|=)\\s*(.+?)\\s*$`),
  );
  if (!opMatch) return html;

  const operator = opMatch[1];
  const literalStr = opMatch[2].trim();
  const literal = parseNumericLiteral(literalStr);

  if (isCondition && literal != null) {
    const result = evaluateCondition(Number(defaultVal), operator, literal);
    if (result != null) {
      html += `<br><span class="urd-tt-dim">Condition: ${defaultVal} ${operator} ${literalStr} → ${result} at game start</span>`;
    }
  } else if (isEffect && literal != null) {
    const result = evaluateEffect(Number(defaultVal), operator, literal);
    if (result != null) {
      html += `<br><span class="urd-tt-dim">After effect: ${defaultVal} ${operator} ${literalStr} → ${result}</span>`;
      if (propDef.min != null && result < (propDef.min as number)) {
        html += `<br><span class="urd-tt-warn">⚠ Below min (${propDef.min})</span>`;
      }
      if (propDef.max != null && result > (propDef.max as number)) {
        html += `<br><span class="urd-tt-warn">⚠ Exceeds max (${propDef.max})</span>`;
      }
    }
  }

  return html;
}

// ─── Dialogue attribution and narrative action ─────────────────────

function appendEntityContextAnnotation(
  html: string,
  entityId: string,
  lineText: string,
  col: number,
): string {
  const atIdx = lineText.lastIndexOf('@', col);
  if (atIdx < 0) return html;
  let idEnd = atIdx + 1;
  while (idEnd < lineText.length && /\w/.test(lineText[idEnd])) idEnd++;
  if (lineText.slice(atIdx + 1, idEnd) !== entityId) return html;

  const afterEntity = lineText[idEnd];
  if (afterEntity === ':') {
    html += `<br><span class="urd-tt-dim">Speaking — dialogue attribution</span>`;
  } else if (afterEntity === ' ') {
    const rest = lineText.slice(idEnd).trim();
    const trimmed = lineText.trimStart();
    if (trimmed.startsWith('? ') || trimmed.startsWith('> ')) return html;
    if (atIdx > 0 && lineText[atIdx - 1] === '[') return html;
    if (rest.length > 0 && !rest.startsWith('.') && !rest.startsWith(']')) {
      html += `<br><span class="urd-tt-dim">Narrative action — stage direction or character action</span>`;
    }
  }
  return html;
}

// ─── Type name tooltip ─────────────────────────────────────────────
// Playground: parsedWorld.types[name] for properties, parsedWorld.entities for instances

function typeNameTooltip(name: string, traits: string[] | undefined, provider: HoverDataProvider | null): string | null {
  let html = `<strong>Type:</strong> ${esc(name)}`;

  if (traits && traits.length > 0) {
    html += `<br><span class="urd-tt-dim">Traits: ${traits.map(esc).join(', ')}</span>`;
  }

  const pw = provider?.getParsedWorld();
  const typeDef = (pw?.types as Record<string, Record<string, unknown>> | undefined)?.[name];
  if (typeDef?.properties && typeof typeDef.properties === 'object') {
    const propNames = Object.keys(typeDef.properties as Record<string, unknown>);
    html += `<br><span class="urd-tt-dim">Properties: ${propNames.map(esc).join(', ')}</span>`;
  }

  // Count entities of this type
  const entities = pw?.entities as Record<string, Record<string, unknown>> | undefined;
  if (entities) {
    const instances = Object.entries(entities)
      .filter(([, e]) => e.type === name)
      .map(([id]) => id);
    if (instances.length > 0) {
      html += `<br><span class="urd-tt-dim">Instances: ${instances.map((id) => `@${esc(id)}`).join(', ')}</span>`;
    }
  }

  return html;
}

// ─── Value literal tooltip ─────────────────────────────────────────

function valueLiteralTooltip(value: string, entityId: string | undefined, property: string | undefined, provider: HoverDataProvider | null): string | null {
  if (!entityId || !property) return null;

  const pw = provider?.getParsedWorld();
  const entity = (pw?.entities as Record<string, Record<string, unknown>> | undefined)?.[entityId];
  if (!entity) return null;
  const typeName = entity.type as string | undefined;
  if (!typeName) return null;

  const prop = (
    (pw?.types as Record<string, Record<string, unknown>> | undefined)?.[typeName]
      ?.properties as Record<string, Record<string, unknown>> | undefined
  )?.[property];
  if (!prop) return null;

  if (prop.type !== 'enum' && !prop.values) return null;
  const validValues: string[] = (prop.values as string[]) ?? [];

  const isValid = validValues.includes(value);
  let html: string;
  if (isValid) {
    html = `<strong>${esc(value)}</strong> — valid value for ${esc(typeName)}.${esc(property)}`;
  } else {
    html = `<strong>${esc(value)}</strong> — <span class="urd-tt-warn">not a valid value for ${esc(typeName)}.${esc(property)}</span>`;
  }
  html += `<br><span class="urd-tt-dim">Valid values: ${validValues.map(esc).join(', ')}</span>`;
  return html;
}

// ─── Static documentation tooltips ─────────────────────────────────

const KEYWORD_DOCS: Record<string, string> = {
  '+': 'Sticky choice — remains available after selection',
  '*': 'One-shot choice — removed after selection',
  '?': 'Condition — following content executes only if this is true',
  '>': 'Effect — modifies entity property values',
  '!': 'Blocked message — shown when the player cannot proceed',
  '->': 'Jump — transfers control to the named section',
  '-> END': 'Built-in: terminates the current conversation',
  '-> RETURN': 'Built-in: returns to the calling section',
  '---': 'Frontmatter delimiter — YAML-like metadata block',
};

function keywordTooltip(token: string): string | null {
  const doc = KEYWORD_DOCS[token];
  if (!doc) return null;
  return `<strong>${esc(token)}</strong><br><span class="urd-tt-dim">${esc(doc)}</span>`;
}

const FRONTMATTER_KEY_DOCS: Record<string, string> = {
  world: 'World identifier — used as the root key in compiled output',
  types: 'Type definitions — declare entity schemas with typed properties',
  entities: 'Entity declarations — instantiate types with optional property overrides',
  import: 'Import — include another .urd.md file in this compilation unit',
};

function frontmatterKeyTooltip(key: string): string | null {
  const doc = FRONTMATTER_KEY_DOCS[key];
  if (!doc) return null;
  return `<strong>${esc(key)}:</strong><br><span class="urd-tt-dim">${esc(doc)}</span>`;
}

const TYPE_CONSTRUCTOR_DOCS: Record<string, { syntax: string; desc: string; examples: string }> = {
  int: { syntax: 'int(min, max)', desc: 'Integer with optional range constraints', examples: 'int, int(0, 100), int(0)' },
  number: { syntax: 'number(min, max)', desc: 'Floating-point with optional range constraints', examples: 'number, number(0.0, 1.0)' },
  string: { syntax: 'string', desc: 'Unicode text value', examples: '"none", "locked", "unknown"' },
  bool: { syntax: 'bool', desc: 'Boolean — true or false (default: false)', examples: 'bool' },
  enum: { syntax: 'enum(value1, value2, ...)', desc: 'Enumerated string — must be one of the listed values', examples: 'enum(friendly, hostile, neutral)' },
  ref: { syntax: 'ref(TypeName)', desc: 'Reference to an entity of the named type', examples: 'ref(Character), ref(Item)' },
  list: { syntax: 'list(element_type)', desc: 'Ordered list of elements', examples: 'list(string), list(ref(Item))' },
  immutable: { syntax: 'immutable', desc: 'Modifier — property cannot be changed by effects', examples: 'immutable' },
};

function typeConstructorTooltip(name: string, range?: string, defaultValue?: string, enumValues?: string): string | null {
  const doc = TYPE_CONSTRUCTOR_DOCS[name];
  if (!doc) return null;
  let html = `<strong>${esc(doc.syntax)}</strong><br>${esc(doc.desc)}`;
  if (range && name !== 'enum') {
    const parts = range.split(',').map((s: string) => s.trim());
    if (parts.length === 2) {
      html += `<br><span class="urd-tt-dim">Range: ${esc(parts[0])} to ${esc(parts[1])}</span>`;
    } else if (parts.length === 1) {
      html += `<br><span class="urd-tt-dim">Min: ${esc(parts[0])}</span>`;
    }
  }
  if (enumValues) {
    html += `<br><span class="urd-tt-dim">Values: ${esc(enumValues)}</span>`;
  }
  if (defaultValue) {
    html += `<br><span class="urd-tt-dim">Default: ${esc(defaultValue)}</span>`;
  }
  if (!range && !defaultValue) {
    html += `<br><span class="urd-tt-dim">Examples: ${esc(doc.examples)}</span>`;
  }
  return html;
}

const TRAIT_DOCS: Record<string, string> = {
  interactable: 'Entities of this type can be the target of dialogue and actions',
  mobile: 'Entities of this type can move between locations via rules',
  container: 'Entities of this type can hold other entities (items, etc.)',
  portable: 'Entities of this type can be picked up, moved, and carried',
};

function traitTooltip(name: string): string | null {
  const doc = TRAIT_DOCS[name];
  if (!doc) return `<strong>Trait:</strong> ${esc(name)}`;
  return `<strong>Trait:</strong> ${esc(name)}<br><span class="urd-tt-dim">${esc(doc)}</span>`;
}

function visibilityTooltip(): string {
  return `<strong>~ Hidden property</strong><br>This property is not visible to the player at runtime.<br><span class="urd-tt-dim">Use reveal @entity.property to make it visible.</span>`;
}

const EFFECT_COMMAND_DOCS: Record<string, { syntax: string; desc: string }> = {
  move: { syntax: 'move @entity -> target', desc: 'Transfers an entity to another entity or location. Target can be player, here, @entity, or a location name.' },
  destroy: { syntax: 'destroy @entity', desc: 'Permanently removes the entity from the world.' },
  reveal: { syntax: 'reveal @entity.property', desc: 'Makes a hidden (~) property visible to the player.' },
};

function effectCommandTooltip(command: string): string | null {
  const doc = EFFECT_COMMAND_DOCS[command];
  if (!doc) return null;
  return `<strong>${esc(doc.syntax)}</strong><br><span class="urd-tt-dim">${esc(doc.desc)}</span>`;
}

const CONDITION_KEYWORD_DOCS: Record<string, string> = {
  'in': 'Containment test — checks if the entity is held by or located in the target',
  'not in': 'Negated containment test — checks the entity is NOT in the target',
  'player': 'The player entity — the implicit protagonist carrying items',
  'here': 'The current location — where the player currently is',
};

function conditionKeywordTooltip(keyword: string): string | null {
  const doc = CONDITION_KEYWORD_DOCS[keyword];
  if (!doc) return null;
  return `<strong>${esc(keyword)}</strong><br><span class="urd-tt-dim">${esc(doc)}</span>`;
}

function conditionCombinatorTooltip(combinator: string): string | null {
  if (combinator === 'any:') {
    return `<strong>any:</strong><br>OR combinator — the following conditions are evaluated as alternatives.<br><span class="urd-tt-dim">At least one must be true for the block to execute.</span>`;
  }
  return null;
}

const RULE_KEYWORD_DOCS: Record<string, string> = {
  rule: 'Rule declaration — defines an NPC behavioural rule triggered by the runtime',
  actor: 'The entity that initiates this rule\'s action',
  action: 'The action verb that triggers this rule',
  selects: 'Select clause — iterates over candidate entities matching the from list',
  target: 'The loop variable — represents each candidate entity being evaluated',
  from: 'The entity pool to select from',
  where: 'Filter condition — candidates must satisfy all where clauses',
};

function ruleKeywordTooltip(keyword: string): string | null {
  const doc = RULE_KEYWORD_DOCS[keyword];
  if (!doc) return null;
  return `<strong>${esc(keyword)}</strong><br><span class="urd-tt-dim">${esc(doc)}</span>`;
}

function ruleNameTooltip(name: string, provider: HoverDataProvider | null): string | null {
  let html = `<strong>Rule:</strong> ${esc(name)}`;
  const defIndex = provider?.getDefinitionIndex();
  const entry = defIndex?.find(
    (d) => d.definition.kind === 'rule' && d.definition.local_name === name,
  );
  if (entry) {
    html += `<br><span class="urd-tt-dim">Defined in ${esc(entry.definition.file_stem ?? 'unknown')}</span>`;
  }
  return html;
}

function sequenceHeadingTooltip(name: string, provider: HoverDataProvider | null): string | null {
  const slug = name.toLowerCase().replace(/\s+/g, '-');
  let html = `<strong>Sequence:</strong> ${esc(slug)}`;
  const defIndex = provider?.getDefinitionIndex();
  const entry = defIndex?.find(
    (d) => d.definition.kind === 'sequence' && d.definition.local_name === name,
  );
  if (entry) {
    html += `<br><span class="urd-tt-dim">Defined in ${esc(entry.definition.file_stem ?? 'unknown')}</span>`;
  } else {
    html += `<br><span class="urd-tt-dim">Defines a multi-phase quest or progression arc.</span>`;
  }
  return html;
}

function phaseHeadingTooltip(name: string, auto: boolean): string | null {
  const slug = name.toLowerCase().replace(/\s+/g, '-');
  let html = `<strong>Phase:</strong> ${esc(slug)}`;
  if (auto) {
    html += `<br><span class="urd-tt-dim">Advance: auto — progresses automatically when conditions are met</span>`;
  } else {
    html += `<br><span class="urd-tt-dim">Advance: manual — requires player to complete an action</span>`;
  }
  return html;
}

const WORLD_KEY_DOCS: Record<string, string> = {
  name: 'World identifier — slug used as the root key in compiled JSON output',
  version: 'Schema version string — included in compiled output for compatibility checking',
  start: 'Starting location — where the player begins',
  entry: 'Entry sequence — the initial quest/progression arc',
  seed: 'Random seed — used by the runtime for deterministic randomisation',
  description: 'Human-readable description of the world',
  author: 'Author attribution',
};

function worldSubKeyTooltip(key: string, value: string | undefined, provider: HoverDataProvider | null): string | null {
  const doc = WORLD_KEY_DOCS[key];
  if (!doc) return null;
  let html = `<strong>${esc(key)}:</strong><br><span class="urd-tt-dim">${esc(doc)}</span>`;

  const defIndex = provider?.getDefinitionIndex();
  if (value && defIndex) {
    if (key === 'start') {
      const entry = defIndex.find(
        (d) => d.definition.kind === 'location' && d.key === `location:${value}`,
      );
      if (entry) {
        const displayName = entry.definition.display_name ?? value;
        html += `<br>Resolves to: <strong>${esc(value)}</strong> (${esc(displayName)})`;
      }
    } else if (key === 'entry') {
      const entry = defIndex.find(
        (d) => d.definition.kind === 'sequence' && (d.definition.local_name === value || d.key === `sequence:${value}`),
      );
      if (entry) {
        const displayName = entry.definition.local_name ?? value;
        html += `<br>Resolves to: <strong>${esc(value)}</strong> (${esc(displayName)})`;
      }
    }
  }

  return html;
}

function commentTooltip(): string {
  return `<strong>//</strong> Comment<br><span class="urd-tt-dim">Ignored by the compiler. Use // to annotate your schema for other authors.</span>`;
}

// ─── Helpers ───────────────────────────────────────────────────────

function parseNumericLiteral(s: string): number | null {
  if (s === 'true') return 1;
  if (s === 'false') return 0;
  const n = Number(s);
  return isNaN(n) ? null : n;
}

function evaluateCondition(left: number, op: string, right: number): string | null {
  switch (op) {
    case '>=': return String(left >= right);
    case '<=': return String(left <= right);
    case '>': return String(left > right);
    case '<': return String(left < right);
    case '==': return String(left === right);
    case '!=': return String(left !== right);
    default: return null;
  }
}

function evaluateEffect(base: number, op: string, value: number): number | null {
  switch (op) {
    case '+': return base + value;
    case '-': return base - value;
    case '*': return base * value;
    case '=': return value;
    default: return null;
  }
}

/**
 * Scan backwards from the given line to find the nearest `# Heading`,
 * then resolve it to a location slug via the DefinitionIndex.
 */
function findEnclosingLocation(view: unknown, fromLine: number, provider: HoverDataProvider | null): string | null {
  if (!view) return null;
  const v = view as { state: { doc: { line(n: number): { text: string } } } };
  const defIndex = provider?.getDefinitionIndex();

  for (let i = fromLine; i >= 1; i--) {
    const text = v.state.doc.line(i).text;
    const match = text.match(/^#\s+(.+)/);
    if (match && !text.startsWith('##')) {
      const displayName = match[1].trim();
      const entry = defIndex?.find(
        (d) => d.definition.kind === 'location' && d.definition.display_name === displayName,
      );
      if (entry) {
        return entry.key.replace(/^location:/, '');
      }
    }
  }
  return null;
}

/** Resolve entity type name — tries raw parsedWorld first, falls back to definitionIndex. */
function resolveEntityType(entityId: string, provider: HoverDataProvider | null): string | null {
  const pw = provider?.getParsedWorld();
  const fromWorld = (pw?.entities as Record<string, Record<string, unknown>> | undefined)?.[entityId]?.type as string | undefined;
  if (fromWorld) return fromWorld;
  // Fall back to definitionIndex
  const defIndex = provider?.getDefinitionIndex();
  const entry = defIndex?.find((d) => d.key === `entity:@${entityId}`);
  return entry?.definition.type_name ?? null;
}

/** Count how many property rows reference this entity (by id or name). */
function countEntityReferences(entityId: string, provider: HoverDataProvider | null): number {
  const entities = provider?.getEntityTable?.();
  const properties = provider?.getPropertyTable?.();
  if (!entities || !properties) return 0;

  const entityRow = entities.find((e) => e.name === entityId || e.id === entityId);
  if (!entityRow) return 0;

  return properties.filter(
    (p) => p.object === entityRow.id || p.object === entityRow.name,
  ).length;
}

function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
