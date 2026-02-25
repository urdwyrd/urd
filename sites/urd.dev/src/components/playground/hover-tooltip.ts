import { hoverTooltip } from '@codemirror/view';
import { identifyReference, getFrontmatterContext, type Reference } from './cursor-resolver';
import { getState, type PlaygroundState } from './playground-state';

/**
 * CodeMirror hover tooltip extension showing rich FactSet data
 * for entities, properties, sections, and locations.
 */
export function urdHoverTooltip() {
  return hoverTooltip((view, pos) => {
    const line = view.state.doc.lineAt(pos);
    const col = pos - line.from;
    const context = getFrontmatterContext(view.state.doc, line.number);
    const ref = identifyReference(line.text, col, context);
    if (!ref) return null;

    const state = getState();
    let content = buildTooltipContent(ref, state, view, line.number);
    if (!content) return null;

    // Presence marker validation: [@entity] shows containment status
    if (ref.kind === 'entity') {
      content = appendPresenceValidation(content, ref.id, line.text, col, view, line.number, state);
      // Dialogue attribution (F14) and narrative action (F15)
      content = appendEntityContextAnnotation(content, ref.id, line.text, col);
    }

    // Default value hints on condition/effect lines
    if (ref.kind === 'entity-property') {
      content = appendDefaultValueHint(content, ref.entityId, ref.property, line.text, state);
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

function buildTooltipContent(
  ref: Reference,
  state: PlaygroundState,
  view?: any,
  lineNumber?: number,
): string | null {
  switch (ref.kind) {
    case 'entity':
      return entityTooltip(ref.id, state);
    case 'entity-property':
      return propertyTooltip(ref.entityId, ref.property, state);
    case 'type-property':
      return typePropertyTooltip(ref.typeName, ref.property, state);
    case 'section-jump':
    case 'section-label':
      return sectionTooltip(ref.name, state, view);
    case 'location-heading':
      return locationTooltip(ref.name, state);
    case 'keyword':
      return keywordTooltip(ref.token);
    case 'frontmatter-key':
      return frontmatterKeyTooltip(ref.key);
    case 'type-constructor':
      return typeConstructorTooltip(ref.name, ref.range, ref.defaultValue, ref.enumValues);
    case 'exit-direction':
      return exitDirectionTooltip(ref.direction, view, lineNumber ?? 0, state);
    case 'exit-destination':
      return exitDestinationTooltip(ref.destinationName, state);
    case 'trait':
      return traitTooltip(ref.name);
    case 'type-name':
      return typeNameTooltip(ref.name, ref.traits, state);
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
      return ruleNameTooltip(ref.name, state);
    case 'sequence-heading':
      return sequenceHeadingTooltip(ref.name, state);
    case 'phase-heading':
      return phaseHeadingTooltip(ref.name, ref.auto);
    case 'world-sub-key':
      return worldSubKeyTooltip(ref.key, ref.value, state);
    case 'exit-jump':
      return exitJumpTooltip(ref.direction, view, lineNumber ?? 0, state);
    case 'value-literal':
      return valueLiteralTooltip(ref.value, ref.entityId, ref.property, state);
    case 'comment':
      return commentTooltip();
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
  // Implicit container property (F13)
  if (property === 'container') {
    const key = `prop:${typeName}.container`;
    const entry = state.definitionIndex?.find((d) => d.key === key);
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

function sectionTooltip(name: string, state: PlaygroundState, view?: any): string | null {
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

  // Content preview — first 5 non-blank lines after the section label
  if (view && entry.span) {
    const preview = getSectionPreview(view, entry.span.start_line);
    if (preview) {
      html += `<br><pre style="margin:4px 0 0;white-space:pre-wrap;color:var(--dim)">${preview}</pre>`;
    }
  }

  return html;
}

function getSectionPreview(view: any, sectionStartLine: number): string | null {
  const doc = view.state.doc;
  const maxLines = doc.lines;
  const collected: string[] = [];
  let hasMore = false;

  for (let i = sectionStartLine + 1; i <= maxLines && collected.length < 5; i++) {
    const text = doc.line(i).text;
    // Stop at next section/location heading
    if (/^(==\s|#\s)/.test(text.trimStart())) {
      break;
    }
    if (text.trim().length > 0) {
      collected.push(text);
    }
  }

  // Check if there are more content lines after the 5 collected
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

// --- Presence marker validation (Tier 2, F6) ---

function appendPresenceValidation(
  html: string,
  entityId: string,
  lineText: string,
  col: number,
  view: any,
  lineNumber: number,
  state: PlaygroundState,
): string {
  // Check if entity is inside [@...] brackets
  const atIdx = lineText.lastIndexOf('@', col);
  if (atIdx < 0) return html;
  if (atIdx < 1 || lineText[atIdx - 1] !== '[') return html;
  const closeBracket = lineText.indexOf(']', atIdx);
  if (closeBracket < 0) return html;

  // Find enclosing location by scanning backwards for # Heading
  const enclosing = findEnclosingLocation(view, lineNumber, state);
  if (!enclosing) return html;

  const location = state.parsedWorld?.locations?.[enclosing];
  const contained = location?.contains?.includes(entityId);

  if (contained) {
    html += `<br><span class="urd-tt-dim">✓ Placed in this location (${esc(enclosing)})</span>`;
  } else {
    html += `<br><span class="urd-tt-warn">⚠ Not contained in this location (${esc(enclosing)})</span>`;
    // Show where the entity actually is
    const locations = state.parsedWorld?.locations;
    if (locations) {
      for (const [locId, loc] of Object.entries(locations) as [string, any][]) {
        if (loc.contains?.includes(entityId)) {
          html += `<br><span class="urd-tt-warn">  Contained in: ${esc(locId)}</span>`;
          break;
        }
      }
    }
  }
  return html;
}

/**
 * Scan backwards from the given line to find the nearest `# Heading`,
 * then resolve it to a location slug via the DefinitionIndex.
 */
function findEnclosingLocation(view: any, fromLine: number, state: PlaygroundState): string | null {
  for (let i = fromLine; i >= 1; i--) {
    const text = view.state.doc.line(i).text;
    const match = text.match(/^#\s+(.+)/);
    if (match && !text.startsWith('##')) {
      const displayName = match[1].trim();
      const entry = state.definitionIndex?.find(
        (d) => d.definition.kind === 'location' && d.definition.display_name === displayName,
      );
      if (entry) {
        return entry.key.replace(/^location:/, '');
      }
    }
  }
  return null;
}

// --- Default value hints (Tier 3, F7) ---

function appendDefaultValueHint(
  html: string,
  entityId: string,
  property: string,
  lineText: string,
  state: PlaygroundState,
): string {
  const trimmed = lineText.trimStart();
  const isCondition = trimmed.startsWith('? ');
  const isEffect = trimmed.startsWith('> ');
  if (!isCondition && !isEffect) return html;

  // Resolve default value
  const entity = state.parsedWorld?.entities?.[entityId];
  if (!entity) return html;
  const typeName = entity.type;
  const typeDef = state.parsedWorld?.types?.[typeName];
  const propDef = typeDef?.properties?.[property];
  if (!propDef) return html;

  // Entity override takes precedence over type default
  const entityOverride = entity.properties?.[property];
  const defaultVal = entityOverride ?? propDef.default;
  if (defaultVal == null) return html;

  const defaultLabel = entityOverride != null ? 'override' : 'default';
  html += `<br><span class="urd-tt-dim">@${esc(entityId)}.${esc(property)} = ${esc(String(defaultVal))} (${defaultLabel})</span>`;

  // Parse the operator and literal from the line
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
      // Range check
      if (propDef.min != null && result < propDef.min) {
        html += `<br><span class="urd-tt-warn">⚠ Below min (${propDef.min})</span>`;
      }
      if (propDef.max != null && result > propDef.max) {
        html += `<br><span class="urd-tt-warn">⚠ Exceeds max (${propDef.max})</span>`;
      }
    }
  }

  return html;
}

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

// --- Static documentation tooltips (Tier 1) ---

const KEYWORD_DOCS: Record<string, string> = {
  '+': 'Sticky choice — remains available after selection',
  '*': 'One-shot choice — removed after selection',
  '?': 'Condition — following content executes only if this is true',
  '>': 'Effect — modifies entity property values',
  '!': 'Blocked message — shown when the player cannot proceed (gate is locked, path is sealed, etc.)',
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
  // Show actual range values (F12)
  if (range && name !== 'enum') {
    const parts = range.split(',').map((s: string) => s.trim());
    if (parts.length === 2) {
      html += `<br><span class="urd-tt-dim">Range: ${esc(parts[0])} to ${esc(parts[1])}</span>`;
    } else if (parts.length === 1) {
      html += `<br><span class="urd-tt-dim">Min: ${esc(parts[0])}</span>`;
    }
  }
  // Show enum values (F12)
  if (enumValues) {
    html += `<br><span class="urd-tt-dim">Values: ${esc(enumValues)}</span>`;
  }
  // Show default value (F12)
  if (defaultValue) {
    html += `<br><span class="urd-tt-dim">Default: ${esc(defaultValue)}</span>`;
  }
  if (!range && !defaultValue) {
    html += `<br><span class="urd-tt-dim">Examples: ${esc(doc.examples)}</span>`;
  }
  return html;
}

// --- Exit direction/destination tooltips (Tier 3, F9) ---

function exitDirectionTooltip(direction: string, view: any, lineNumber: number, state: PlaygroundState): string | null {
  const enclosing = findEnclosingLocation(view, lineNumber, state);
  if (!enclosing) return null;

  const location = state.parsedWorld?.locations?.[enclosing];
  const exits = location?.exits;
  if (!exits || typeof exits !== 'object' || Object.keys(exits).length === 0) return null;

  // Get display name from definition index
  const defEntry = state.definitionIndex?.find(
    (d) => d.definition.kind === 'location' && d.key === `location:${enclosing}`,
  );
  const displayName = defEntry?.definition?.display_name ?? enclosing.replace(/-/g, ' ');

  let html = `<strong>Exits from ${esc(displayName)}</strong>`;

  const facts = state.result?.facts;
  for (const [dir, exitData] of Object.entries(exits) as [string, any][]) {
    const dest = exitData.to ?? '?';
    let line = `  ${esc(dir)} → ${esc(dest.replace(/-/g, ' '))}`;
    // Check conditional status from FactSet
    if (facts?.exits) {
      const exitFact = facts.exits.find(
        (e: any) => e.from_location === enclosing && e.exit_name === dir,
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

function exitDestinationTooltip(destinationName: string, state: PlaygroundState): string | null {
  const locations = state.parsedWorld?.locations;
  if (!locations) return null;

  // Look up slug via definition index (display name → slug)
  let slug: string | null = null;
  const defEntry = state.definitionIndex?.find(
    (d) => d.definition.kind === 'location' && d.definition.display_name === destinationName,
  );
  if (defEntry) {
    slug = defEntry.key.replace(/^location:/, '');
  } else {
    // Fallback: try slugified form of the display name
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

  // Description
  if (loc.description) {
    html += `<br><span class="urd-tt-dim">${esc(loc.description)}</span>`;
  }

  // Exits (map: direction → { to: slug })
  const exits = loc.exits;
  if (exits && typeof exits === 'object' && Object.keys(exits).length > 0) {
    const exitList = Object.entries(exits as Record<string, any>)
      .map(([dir, data]) => `${esc(dir)} → ${esc((data.to ?? '?').replace(/-/g, ' '))}`)
      .join(', ');
    html += `<br><span class="urd-tt-dim">Exits: ${exitList}</span>`;
  }

  // Contains
  if (loc.contains?.length > 0) {
    const entityList = loc.contains.map((id: string) => `@${esc(id)}`).join(', ');
    html += `<br><span class="urd-tt-dim">Contains: ${entityList}</span>`;
  }

  return html;
}

// --- Dialogue attribution and narrative action (F14, F15) ---

function appendEntityContextAnnotation(
  html: string,
  entityId: string,
  lineText: string,
  col: number,
): string {
  // Find the entity span in the line
  const atIdx = lineText.lastIndexOf('@', col);
  if (atIdx < 0) return html;
  let idEnd = atIdx + 1;
  while (idEnd < lineText.length && /\w/.test(lineText[idEnd])) idEnd++;
  if (lineText.slice(atIdx + 1, idEnd) !== entityId) return html;

  const afterEntity = lineText[idEnd];
  if (afterEntity === ':') {
    // Dialogue attribution: @entity: text
    html += `<br><span class="urd-tt-dim">Speaking — dialogue attribution</span>`;
  } else if (afterEntity === ' ') {
    // Check it's not a property access, presence marker, or structural token
    const rest = lineText.slice(idEnd).trim();
    // Not on a condition/effect line marker
    const trimmed = lineText.trimStart();
    if (trimmed.startsWith('? ') || trimmed.startsWith('> ')) return html;
    // Not inside brackets
    if (atIdx > 0 && lineText[atIdx - 1] === '[') return html;
    // Has text after — narrative action
    if (rest.length > 0 && !rest.startsWith('.') && !rest.startsWith(']')) {
      html += `<br><span class="urd-tt-dim">Narrative action — stage direction or character action</span>`;
    }
  }
  return html;
}

// --- Type name tooltip ---

function typeNameTooltip(name: string, traits: string[] | undefined, state: PlaygroundState): string | null {
  let html = `<strong>Type:</strong> ${esc(name)}`;

  if (traits && traits.length > 0) {
    html += `<br><span class="urd-tt-dim">Traits: ${traits.map(esc).join(', ')}</span>`;
  }

  // Count properties from parsedWorld.types
  const typeDef = state.parsedWorld?.types?.[name];
  if (typeDef?.properties) {
    const propNames = Object.keys(typeDef.properties);
    html += `<br><span class="urd-tt-dim">Properties: ${propNames.map(esc).join(', ')}</span>`;
  }

  // Count entities of this type
  const entities = state.parsedWorld?.entities;
  if (entities) {
    const instances = Object.entries(entities)
      .filter(([, e]: [string, any]) => e.type === name)
      .map(([id]) => id);
    if (instances.length > 0) {
      html += `<br><span class="urd-tt-dim">Instances: ${instances.map((id) => `@${esc(id)}`).join(', ')}</span>`;
    }
  }

  return html;
}

// --- Trait tooltip (F1) ---

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

// --- Hidden visibility tooltip (F2) ---

function visibilityTooltip(): string {
  return `<strong>~ Hidden property</strong><br>This property is not visible to the player at runtime.<br><span class="urd-tt-dim">Use reveal @entity.property to make it visible.</span>`;
}

// --- Effect command tooltip (F3) ---

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

// --- Condition keyword tooltip (F4) ---

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

// --- Condition combinator tooltip (F5) ---

function conditionCombinatorTooltip(combinator: string): string | null {
  if (combinator === 'any:') {
    return `<strong>any:</strong><br>OR combinator — the following conditions are evaluated as alternatives.<br><span class="urd-tt-dim">At least one must be true for the block to execute.</span>`;
  }
  return null;
}

// --- Rule keyword tooltip (F6) ---

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

// --- Rule name tooltip (F6) ---

function ruleNameTooltip(name: string, state: PlaygroundState): string | null {
  let html = `<strong>Rule:</strong> ${esc(name)}`;

  // Look up in DefinitionIndex
  const entry = state.definitionIndex?.find(
    (d) => d.definition.kind === 'rule' && d.definition.local_name === name,
  );
  if (entry) {
    html += `<br><span class="urd-tt-dim">Defined in ${esc(entry.definition.file_stem ?? 'unknown')}</span>`;
  }

  return html;
}

// --- Sequence heading tooltip (F7) ---

function sequenceHeadingTooltip(name: string, state: PlaygroundState): string | null {
  const slug = name.toLowerCase().replace(/\s+/g, '-');
  let html = `<strong>Sequence:</strong> ${esc(slug)}`;

  const entry = state.definitionIndex?.find(
    (d) => d.definition.kind === 'sequence' && d.definition.local_name === name,
  );
  if (entry) {
    html += `<br><span class="urd-tt-dim">Defined in ${esc(entry.definition.file_stem ?? 'unknown')}</span>`;
  } else {
    html += `<br><span class="urd-tt-dim">Defines a multi-phase quest or progression arc.</span>`;
  }

  return html;
}

// --- Phase heading tooltip (F8) ---

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

// --- World sub-key tooltip (F9) ---

const WORLD_KEY_DOCS: Record<string, string> = {
  name: 'World identifier — slug used as the root key in compiled JSON output',
  version: 'Schema version string — included in compiled output for compatibility checking',
  start: 'Starting location — where the player begins',
  entry: 'Entry sequence — the initial quest/progression arc',
  seed: 'Random seed — used by the runtime for deterministic randomisation',
  description: 'Human-readable description of the world',
  author: 'Author attribution',
};

function worldSubKeyTooltip(key: string, value: string | undefined, state: PlaygroundState): string | null {
  const doc = WORLD_KEY_DOCS[key];
  if (!doc) return null;
  let html = `<strong>${esc(key)}:</strong><br><span class="urd-tt-dim">${esc(doc)}</span>`;

  // Resolve start → location, entry → sequence (F9)
  if (value && state.definitionIndex) {
    if (key === 'start') {
      const entry = state.definitionIndex.find(
        (d) => d.definition.kind === 'location' && d.key === `location:${value}`,
      );
      if (entry) {
        const displayName = entry.definition.display_name ?? value;
        html += `<br>Resolves to: <strong>${esc(value)}</strong> (${esc(displayName)})`;
      }
    } else if (key === 'entry') {
      const entry = state.definitionIndex.find(
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

// --- Exit-jump tooltip (F10) ---

function exitJumpTooltip(direction: string, view: any, lineNumber: number, state: PlaygroundState): string | null {
  let html = `<strong>Exit jump:</strong> ${esc(direction)}`;
  html += `<br><span class="urd-tt-dim">Navigates via the exit named "${esc(direction)}" from the current location.</span>`;

  // Resolve destination from enclosing location's exits
  const enclosing = findEnclosingLocation(view, lineNumber, state);
  if (enclosing) {
    const location = state.parsedWorld?.locations?.[enclosing];
    if (location?.exits) {
      const exit = location.exits.find((e: any) => e.direction === direction);
      if (exit?.destination) {
        html += `<br>Resolves to: <strong>${esc(exit.destination)}</strong>`;
      }
    }
  }

  return html;
}

// --- Value literal tooltip (F16) ---

function valueLiteralTooltip(value: string, entityId: string | undefined, property: string | undefined, state: PlaygroundState): string | null {
  if (!entityId || !property) return null;

  const entity = state.parsedWorld?.entities?.[entityId];
  if (!entity) return null;
  const typeName = entity.type;
  const prop = state.parsedWorld?.types?.[typeName]?.properties?.[property];
  if (!prop) return null;

  // Only show for enum properties
  if (prop.type !== 'enum' && !prop.values) return null;
  const validValues: string[] = prop.values ?? [];

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

// --- Comment tooltip (F17) ---

function commentTooltip(): string {
  return `<strong>//</strong> Comment<br><span class="urd-tt-dim">Ignored by the compiler. Use // to annotate your schema for other authors.</span>`;
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
