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
      return typeConstructorTooltip(ref.name);
    case 'exit-direction':
      return exitDirectionTooltip(ref.direction, view, lineNumber ?? 0, state);
    case 'exit-destination':
      return exitDestinationTooltip(ref.destinationName, state);
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
  string: { syntax: 'string', desc: 'Text value', examples: 'string' },
  bool: { syntax: 'bool', desc: 'Boolean — true or false (default: false)', examples: 'bool' },
  enum: { syntax: 'enum(value1, value2, ...)', desc: 'Enumerated string — must be one of the listed values', examples: 'enum(friendly, hostile, neutral)' },
  ref: { syntax: 'ref(TypeName)', desc: 'Reference to an entity of the named type', examples: 'ref(Character), ref(Item)' },
  list: { syntax: 'list(element_type)', desc: 'Ordered list of elements', examples: 'list(string), list(ref(Item))' },
  immutable: { syntax: 'immutable', desc: 'Modifier — property cannot be changed by effects', examples: 'immutable' },
};

function typeConstructorTooltip(name: string): string | null {
  const doc = TYPE_CONSTRUCTOR_DOCS[name];
  if (!doc) return null;
  return `<strong>${esc(doc.syntax)}</strong><br>${esc(doc.desc)}<br><span class="urd-tt-dim">Examples: ${esc(doc.examples)}</span>`;
}

// --- Exit direction/destination tooltips (Tier 3, F9) ---

function exitDirectionTooltip(direction: string, view: any, lineNumber: number, state: PlaygroundState): string | null {
  const enclosing = findEnclosingLocation(view, lineNumber, state);
  if (!enclosing) return null;

  const location = state.parsedWorld?.locations?.[enclosing];
  if (!location?.exits || location.exits.length === 0) return null;

  let html = `<strong>Exits from ${esc(location.display_name ?? enclosing)}</strong>`;

  const facts = state.result?.facts;
  for (const exit of location.exits) {
    const dir = exit.direction ?? '?';
    const dest = exit.destination ?? '?';
    let line = `  ${esc(dir)} → ${esc(dest)}`;
    // Check conditional status from FactSet
    if (facts?.exits) {
      const exitFact = facts.exits.find(
        (e: any) => e.from === enclosing && e.direction === dir,
      );
      if (exitFact?.is_conditional) {
        line += ' (conditional)';
      }
    }
    const isCurrent = dir === direction;
    html += `<br><span class="${isCurrent ? 'urd-tt-dim' : 'urd-tt-dim'}" style="${isCurrent ? 'font-weight:600' : ''}">${line}</span>`;
  }

  return html;
}

function exitDestinationTooltip(destinationName: string, state: PlaygroundState): string | null {
  // Find the location by display name or slug
  const locations = state.parsedWorld?.locations;
  if (!locations) return null;

  let slug: string | null = null;
  let loc: any = null;
  for (const [id, l] of Object.entries(locations) as [string, any][]) {
    if (l.display_name === destinationName || id === destinationName) {
      slug = id;
      loc = l;
      break;
    }
  }
  if (!slug || !loc) return null;

  let html = `<strong>Location</strong>: ${esc(slug)}`;

  // Exits
  if (loc.exits?.length > 0) {
    const exitList = loc.exits
      .map((e: any) => `${esc(e.direction ?? '?')} → ${esc(e.destination ?? '?')}`)
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
