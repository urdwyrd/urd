/**
 * Port of packages/lsp/src/cursor.rs — identifies the Urd construct
 * under the cursor given a line of text and a character column.
 */

export type Reference =
  | { kind: 'entity'; id: string }
  | { kind: 'entity-property'; entityId: string; property: string }
  | { kind: 'type-property'; typeName: string; property: string }
  | { kind: 'section-jump'; name: string }
  | { kind: 'section-label'; name: string }
  | { kind: 'location-heading'; name: string }
  | { kind: 'keyword'; token: string }
  | { kind: 'frontmatter-key'; key: string }
  | { kind: 'type-constructor'; name: string; range?: string; defaultValue?: string; enumValues?: string }
  | { kind: 'exit-direction'; direction: string }
  | { kind: 'exit-destination'; destinationName: string }
  | { kind: 'trait'; name: string }
  | { kind: 'visibility-prefix'; visibility: 'hidden' }
  | { kind: 'effect-command'; command: string }
  | { kind: 'condition-keyword'; keyword: string }
  | { kind: 'condition-combinator'; combinator: string }
  | { kind: 'rule-keyword'; keyword: string }
  | { kind: 'rule-name'; name: string }
  | { kind: 'sequence-heading'; name: string }
  | { kind: 'phase-heading'; name: string; auto: boolean }
  | { kind: 'world-sub-key'; key: string; value?: string }
  | { kind: 'exit-jump'; direction: string }
  | { kind: 'value-literal'; value: string; entityId?: string; property?: string }
  | { kind: 'comment' }
  | { kind: 'type-name'; name: string; traits?: string[] };

/**
 * Identify the Urd reference at the given column in a line of text.
 * Column is 0-indexed character offset from line start.
 *
 * When `context` is provided, frontmatter-aware reference kinds
 * (frontmatter-key, type-constructor) can be detected.
 */
export function identifyReference(
  line: string,
  col: number,
  context?: ReferenceContext,
): Reference | null {
  const trimmed = line.trimStart();
  const trimOffset = line.length - trimmed.length;

  // 1. Comments: // anywhere
  const commentIdx = line.indexOf('//');
  if (commentIdx >= 0 && col >= commentIdx && col < commentIdx + 2) {
    return { kind: 'comment' };
  }

  // 2. Section label: == name
  if (trimmed.startsWith('== ') || trimmed.startsWith('==')) {
    const prefixLen = trimmed.startsWith('== ') ? 3 : 2;
    const name = trimmed.slice(prefixLen).trim();
    if (name.length > 0) {
      return { kind: 'section-label', name };
    }
  }

  // 3. Phase heading: ### (before ## and # checks)
  if (trimmed.startsWith('### ')) {
    let name = trimmed.slice(4).trim();
    const auto = /\(auto\)\s*$/.test(name);
    if (auto) name = name.replace(/\s*\(auto\)\s*$/, '').trim();
    if (name.length > 0) {
      return { kind: 'phase-heading', name, auto };
    }
  }

  // 4. Sequence heading: ## (but not ### which was caught above)
  if (trimmed.startsWith('## ') && !trimmed.startsWith('### ')) {
    const name = trimmed.slice(3).trim();
    if (name.length > 0) {
      return { kind: 'sequence-heading', name };
    }
  }

  // 5. Location heading: # (single # only, not ##)
  if (trimmed.startsWith('# ') && !trimmed.startsWith('## ')) {
    const name = trimmed.slice(2).trim();
    if (name.length > 0) {
      return { kind: 'location-heading', name };
    }
  }

  // 6. Frontmatter delimiter: ---
  if (/^---\s*$/.test(trimmed) && col >= trimOffset && col <= trimOffset + 3) {
    return { kind: 'keyword', token: '---' };
  }

  // 7. Rule declaration: `rule name:`
  if (/^rule\s+\w+/.test(trimmed)) {
    const ruleMatch = trimmed.match(/^(rule)\s+(\w+)/);
    if (ruleMatch) {
      const ruleKwStart = trimOffset;
      const ruleKwEnd = trimOffset + 4;
      if (col >= ruleKwStart && col < ruleKwEnd) {
        return { kind: 'rule-keyword', keyword: 'rule' };
      }
      const nameStart = trimOffset + ruleMatch[0].indexOf(ruleMatch[2]);
      const nameEnd = nameStart + ruleMatch[2].length;
      if (col >= nameStart && col < nameEnd) {
        return { kind: 'rule-name', name: ruleMatch[2] };
      }
    }
  }

  // 8. Section jump: -> target (check for built-ins and exit: first)
  const arrowIdx = line.indexOf('->');
  if (arrowIdx >= 0 && col >= arrowIdx) {
    const afterArrow = line.slice(arrowIdx + 2).trim();
    // Case-insensitive END/RETURN (F11)
    if (afterArrow.toUpperCase() === 'END' || afterArrow.toUpperCase() === 'RETURN') {
      return { kind: 'keyword', token: `-> ${afterArrow.toUpperCase()}` };
    }
    // Exit-jump: -> exit:direction (F10)
    if (afterArrow.startsWith('exit:')) {
      const direction = afterArrow.slice(5).trim();
      if (direction.length > 0) {
        return { kind: 'exit-jump', direction };
      }
    }
    // Regular section jump
    const arrowResult = findSectionJump(line, col);
    if (arrowResult) return arrowResult;
    // Cursor on the -> itself but no target resolved
    if (col >= arrowIdx && col < arrowIdx + 2) {
      return { kind: 'keyword', token: '->' };
    }
  }

  // 9. Line-start structural markers: + * ? >
  const keywordResult = findLineStartKeyword(trimmed, col, trimOffset);
  if (keywordResult) return keywordResult;

  // 10. Effect commands and condition keywords on ? and > lines
  if (!context?.inFrontmatter) {
    const effectCmdResult = findEffectCommand(trimmed, col, trimOffset);
    if (effectCmdResult) return effectCmdResult;

    const condKwResult = findConditionKeyword(line, trimmed, col, trimOffset);
    if (condKwResult) return condKwResult;

    // Value literals on condition/effect lines
    const valResult = findValueLiteral(line, trimmed, col);
    if (valResult) return valResult;
  }

  // 11. Rule block keywords
  if (context?.inRuleBlock) {
    const ruleKwResult = findRuleKeyword(trimmed, col, trimOffset);
    if (ruleKwResult) return ruleKwResult;
  }

  // 12. Frontmatter-aware references
  if (context?.inFrontmatter) {
    const fmKeyResult = findFrontmatterKey(trimmed, col, trimOffset);
    if (fmKeyResult) return fmKeyResult;

    // World sub-keys (F9)
    if (context.inWorldBlock) {
      const worldResult = findWorldSubKey(trimmed, col, trimOffset);
      if (worldResult) return worldResult;
    }

    // Trait markers (F1)
    const traitResult = findTraitMarker(trimmed, col, trimOffset);
    if (traitResult) return traitResult;

    // Type name — definition lines and entity declaration lines
    const typeNameResult = findTypeName(trimmed, col, trimOffset);
    if (typeNameResult) return typeNameResult;

    // Hidden visibility prefix (F2)
    if (context.inTypeBlock) {
      const visResult = findVisibilityPrefix(trimmed, col, trimOffset);
      if (visResult) return visResult;
    }

    const typeConResult = findTypeConstructor(trimmed, col, trimOffset, context);
    if (typeConResult) return typeConResult;
  }

  // 13. Exit direction/destination: `direction: Destination Name` (body only)
  if (!context?.inFrontmatter) {
    const exitResult = findExitReference(trimmed, col, trimOffset);
    if (exitResult) return exitResult;
  }

  // 14. Entity / entity.property reference
  const entityResult = findEntityReference(line, col);
  if (entityResult) return entityResult;

  // 15. Type.property reference (uppercase start before dot)
  const typeResult = findTypeProperty(line, col);
  if (typeResult) return typeResult;

  return null;
}

function findSectionJump(line: string, col: number): Reference | null {
  const arrowIdx = line.indexOf('->');
  if (arrowIdx < 0) return null;

  // Extract target: skip whitespace after ->
  let targetStart = arrowIdx + 2;
  while (targetStart < line.length && line[targetStart] === ' ') targetStart++;

  let targetEnd = targetStart;
  const firstChar = line[targetStart];
  // Strip leading @ if present
  const skipAt = firstChar === '@' ? 1 : 0;
  const wordStart = targetStart + skipAt;
  targetEnd = wordStart;
  while (targetEnd < line.length && isWordChar(line[targetEnd])) targetEnd++;

  if (wordStart >= targetEnd) return null;

  // Check if cursor is within arrow..target span
  if (col >= arrowIdx && col < targetEnd) {
    const name = line.slice(wordStart, targetEnd);
    return { kind: 'section-jump', name };
  }

  return null;
}

function findEntityReference(line: string, col: number): Reference | null {
  // Iterate through all @ positions
  for (let i = 0; i < line.length; i++) {
    if (line[i] !== '@') continue;

    // Extract entity_id: alphanumeric + underscore
    let idEnd = i + 1;
    while (idEnd < line.length && isWordChar(line[idEnd])) idEnd++;

    if (idEnd === i + 1) continue; // No id after @

    const entityId = line.slice(i + 1, idEnd);

    // Check for dot-property
    if (idEnd < line.length && line[idEnd] === '.') {
      let propEnd = idEnd + 1;
      while (propEnd < line.length && isWordChar(line[propEnd])) propEnd++;

      if (propEnd > idEnd + 1) {
        const property = line.slice(idEnd + 1, propEnd);
        // Cursor anywhere in @entity.property span
        if (col >= i && col < propEnd) {
          return { kind: 'entity-property', entityId, property };
        }
        continue;
      }
    }

    // No property — check cursor in @entity span
    if (col >= i && col < idEnd) {
      return { kind: 'entity', id: entityId };
    }
  }

  return null;
}

function findTypeProperty(line: string, col: number): Reference | null {
  for (let i = 0; i < line.length; i++) {
    if (line[i] !== '.') continue;

    // Extract word before dot
    let typeStart = i - 1;
    while (typeStart >= 0 && isWordChar(line[typeStart])) typeStart--;
    typeStart++;

    // Must start with uppercase (distinguishes Type.prop from @entity.prop)
    if (typeStart >= i) continue;
    const firstChar = line[typeStart];
    if (firstChar < 'A' || firstChar > 'Z') continue;

    // Must not be preceded by @ (that's entity.property, handled above)
    if (typeStart > 0 && line[typeStart - 1] === '@') continue;

    const typeName = line.slice(typeStart, i);

    // Extract word after dot
    let propEnd = i + 1;
    while (propEnd < line.length && isWordChar(line[propEnd])) propEnd++;

    if (propEnd === i + 1) continue;

    const property = line.slice(i + 1, propEnd);

    if (col >= typeStart && col < propEnd) {
      return { kind: 'type-property', typeName, property };
    }
  }

  return null;
}

function isWordChar(ch: string): boolean {
  return (ch >= 'a' && ch <= 'z') ||
    (ch >= 'A' && ch <= 'Z') ||
    (ch >= '0' && ch <= '9') ||
    ch === '_';
}

// --- Context for frontmatter-aware detection ---

export interface ReferenceContext {
  inFrontmatter: boolean;
  /** Whether cursor is inside a `types:` block (indented under a type name). */
  inTypeBlock?: boolean;
  /** Whether cursor is indented under `world:`. */
  inWorldBlock?: boolean;
  /** Whether cursor is inside a `rule name:` block (body content). */
  inRuleBlock?: boolean;
}

/**
 * Determine frontmatter context from a CodeMirror document and position.
 * Scans backwards from the given line to find `---` delimiters.
 */
export function getFrontmatterContext(
  doc: { line(n: number): { text: string }; lines: number },
  lineNumber: number,
): ReferenceContext {
  let delimCount = 0;
  let inTypeBlock = false;
  let inWorldBlock = false;
  let currentTopKey = '';

  for (let i = 1; i <= Math.min(lineNumber, doc.lines); i++) {
    const text = doc.line(i).text.trimEnd();
    if (/^---\s*$/.test(text)) {
      delimCount++;
      if (delimCount >= 2) break;
      continue;
    }
    if (delimCount === 1) {
      // Track top-level frontmatter key
      if (/^\w/.test(text) && text.includes(':')) {
        currentTopKey = text.match(/^(\w+)/)?.[1] ?? '';
      }
      // If cursor line is indented, check which block we're in
      if (i === lineNumber && /^\s{2,}/.test(doc.line(i).text)) {
        if (currentTopKey === 'types') inTypeBlock = true;
        if (currentTopKey === 'world') inWorldBlock = true;
      }
    }
  }

  const inFrontmatter = delimCount === 1 && lineNumber > 1;

  // Rule block detection — scan backwards from current line (body only)
  let inRuleBlock = false;
  if (!inFrontmatter) {
    for (let i = lineNumber; i >= 1; i--) {
      const text = doc.line(i).text;
      // Blank line breaks the rule block
      if (text.trim() === '' && i < lineNumber) break;
      // Found a rule declaration at indent 0
      if (/^rule\s+\w+/.test(text)) {
        if (i < lineNumber) inRuleBlock = true;
        break;
      }
      // Non-indented non-rule line breaks the search
      if (i < lineNumber && /^\S/.test(text) && !/^rule\s/.test(text)) break;
    }
  }

  return { inFrontmatter, inTypeBlock, inWorldBlock, inRuleBlock };
}

// --- Line-start keyword detection ---

function findLineStartKeyword(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Only match markers at the start of the (trimmed) line
  const markers: [RegExp, string][] = [
    [/^\+\s/, '+'],
    [/^\*\s/, '*'],
    [/^\?\s/, '?'],
    [/^>\s/, '>'],
    [/^!\s/, '!'],
  ];
  for (const [pattern, token] of markers) {
    if (pattern.test(trimmed) && col >= trimOffset && col < trimOffset + 1) {
      return { kind: 'keyword', token };
    }
  }
  return null;
}

// --- Frontmatter key detection ---

function findFrontmatterKey(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Top-level frontmatter key: word at indent 0 followed by colon
  if (trimOffset > 0) return null; // Must be at indent level 0
  const match = trimmed.match(/^(\w+)\s*:/);
  if (!match) return null;
  const key = match[1];
  const keyEnd = key.length;
  if (col >= 0 && col <= keyEnd) {
    return { kind: 'frontmatter-key', key };
  }
  return null;
}

// --- Type constructor detection ---

const TYPE_CONSTRUCTORS = new Set([
  'int', 'number', 'string', 'bool', 'enum', 'ref', 'list', 'immutable',
]);

function findTypeConstructor(
  trimmed: string,
  col: number,
  trimOffset: number,
  context: ReferenceContext,
): Reference | null {
  if (!context.inTypeBlock) return null;
  // Property line: `prop: type(args) = default` or `~prop: type(args) = default`
  const match = trimmed.match(/^~?(\w+)\s*:\s*(\w+)/);
  if (!match) return null;
  const typeCon = match[2];
  if (!TYPE_CONSTRUCTORS.has(typeCon)) return null;
  // Find the column range of the constructor in the original line
  const typeConStart = trimOffset + match.index! + match[0].indexOf(typeCon);
  const typeConEnd = typeConStart + typeCon.length;
  if (col >= typeConStart && col < typeConEnd) {
    // Extract range arguments: int(0, 100) or enum(a, b, c)
    const argsMatch = trimmed.match(new RegExp(`${typeCon}\\(([^)]*)\\)`));
    const range = argsMatch ? argsMatch[1] : undefined;
    // Extract default value: = value
    const defMatch = trimmed.match(/=\s*(.+?)$/);
    const defaultValue = defMatch ? defMatch[1].trim() : undefined;
    // For enum, extract values as enumValues
    const enumValues = (typeCon === 'enum' && range) ? range : undefined;
    return { kind: 'type-constructor', name: typeCon, range, defaultValue, enumValues };
  }
  return null;
}

// --- Exit direction/destination detection ---

// --- Effect command detection (F3) ---

const EFFECT_COMMANDS = new Set(['move', 'destroy', 'reveal']);

function findEffectCommand(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Effect lines start with `>`
  if (!trimmed.startsWith('> ')) return null;
  const afterMarker = trimmed.slice(2).trimStart();
  const afterMarkerOffset = trimOffset + 2 + (trimmed.slice(2).length - afterMarker.length);

  const wordMatch = afterMarker.match(/^(\w+)/);
  if (!wordMatch) return null;
  const command = wordMatch[1].toLowerCase();
  if (!EFFECT_COMMANDS.has(command)) return null;

  const cmdStart = afterMarkerOffset;
  const cmdEnd = cmdStart + wordMatch[1].length;
  if (col >= cmdStart && col < cmdEnd) {
    return { kind: 'effect-command', command };
  }
  return null;
}

// --- Condition keyword detection (F4, F5) ---

function findConditionKeyword(line: string, trimmed: string, col: number, _trimOffset: number): Reference | null {
  // Condition lines start with `?`, but `player` and `here` also appear on effect lines (`>`)
  const isCondition = trimmed.startsWith('? ');
  const isEffect = trimmed.startsWith('> ');
  if (!isCondition && !isEffect) return null;

  // `any:` — OR combinator (condition lines only, F5)
  if (isCondition) {
    const anyMatch = trimmed.match(/^\?\s+(any\s*:)/);
    if (anyMatch) {
      const anyStart = line.indexOf(anyMatch[1]);
      if (anyStart >= 0 && col >= anyStart && col < anyStart + anyMatch[1].length) {
        return { kind: 'condition-combinator', combinator: 'any:' };
      }
    }
  }

  // `not in` — must check before `in` to avoid partial match
  const notInRegex = /\bnot\s+in\b/g;
  let notInMatch: RegExpExecArray | null;
  while ((notInMatch = notInRegex.exec(line)) !== null) {
    const start = notInMatch.index;
    const end = start + notInMatch[0].length;
    if (col >= start && col < end) {
      return { kind: 'condition-keyword', keyword: 'not in' };
    }
  }

  // `in` — standalone word
  const inRegex = /\bin\b/g;
  let inMatch: RegExpExecArray | null;
  while ((inMatch = inRegex.exec(line)) !== null) {
    // Skip if part of `not in`
    const before = line.slice(Math.max(0, inMatch.index - 4), inMatch.index);
    if (/not\s$/.test(before)) continue;
    const start = inMatch.index;
    const end = start + 2;
    if (col >= start && col < end) {
      return { kind: 'condition-keyword', keyword: 'in' };
    }
  }

  // `player` keyword
  const playerRegex = /\bplayer\b/g;
  let playerMatch: RegExpExecArray | null;
  while ((playerMatch = playerRegex.exec(line)) !== null) {
    // Skip if preceded by @ (that's an entity reference)
    if (playerMatch.index > 0 && line[playerMatch.index - 1] === '@') continue;
    const start = playerMatch.index;
    const end = start + 6;
    if (col >= start && col < end) {
      return { kind: 'condition-keyword', keyword: 'player' };
    }
  }

  // `here` keyword
  const hereRegex = /\bhere\b/g;
  let hereMatch: RegExpExecArray | null;
  while ((hereMatch = hereRegex.exec(line)) !== null) {
    if (hereMatch.index > 0 && line[hereMatch.index - 1] === '@') continue;
    const start = hereMatch.index;
    const end = start + 4;
    if (col >= start && col < end) {
      return { kind: 'condition-keyword', keyword: 'here' };
    }
  }

  return null;
}

// --- Value literal detection (F16) ---

function findValueLiteral(line: string, trimmed: string, col: number): Reference | null {
  const isCondition = trimmed.startsWith('? ');
  const isEffect = trimmed.startsWith('> ');
  if (!isCondition && !isEffect) return null;

  // Pattern: @entity.property operator value
  const valMatch = line.match(/@(\w+)\.(\w+)\s*(?:>=|<=|==|!=|>|<|\+|-|\*|=)\s*(\S+)\s*$/);
  if (!valMatch) return null;

  const entityId = valMatch[1];
  const property = valMatch[2];
  const value = valMatch[3];
  const valueStart = line.lastIndexOf(value);
  const valueEnd = valueStart + value.length;

  if (col >= valueStart && col < valueEnd) {
    return { kind: 'value-literal', value, entityId, property };
  }
  return null;
}

// --- Rule keyword detection (F6) ---

const RULE_KEYWORD_DOCS_KEYS = new Set([
  'actor', 'action', 'selects', 'from', 'where', 'target',
]);

function findRuleKeyword(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Inside a rule block — match keywords at their positions
  // `actor:` — special because it has a colon
  const actorMatch = trimmed.match(/^\s*(actor)\s*:/);
  if (actorMatch) {
    const kw = actorMatch[1];
    const kwStart = trimOffset + trimmed.indexOf(kw);
    const kwEnd = kwStart + kw.length;
    if (col >= kwStart && col < kwEnd) {
      return { kind: 'rule-keyword', keyword: 'actor' };
    }
  }

  // Other keywords: scan for word tokens
  for (const kw of RULE_KEYWORD_DOCS_KEYS) {
    if (kw === 'actor') continue; // handled above
    const regex = new RegExp(`\\b${kw}\\b`, 'g');
    let m: RegExpExecArray | null;
    const fullLine = ' '.repeat(trimOffset) + trimmed;
    while ((m = regex.exec(fullLine)) !== null) {
      if (col >= m.index && col < m.index + kw.length) {
        return { kind: 'rule-keyword', keyword: kw };
      }
    }
  }

  return null;
}

// --- World sub-key detection (F9) ---

const WORLD_SUB_KEYS = new Set([
  'name', 'version', 'start', 'entry', 'seed', 'description', 'author',
]);

function findWorldSubKey(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Indented line under world: — `  key: value`
  if (trimOffset < 2) return null; // Must be indented
  const kvMatch = trimmed.match(/^(\w+)\s*:\s*(.*)$/);
  if (!kvMatch) return null;
  const key = kvMatch[1];
  if (!WORLD_SUB_KEYS.has(key)) return null;
  const keyStart = trimOffset;
  const keyEnd = trimOffset + key.length;
  if (col >= keyStart && col <= keyEnd) {
    const value = kvMatch[2].replace(/^["']|["']$/g, '').trim() || undefined;
    return { kind: 'world-sub-key', key, value };
  }
  return null;
}

// --- Trait marker detection (F1) ---

function findTraitMarker(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Type definition line: `TypeName [trait1, trait2]:` or `  TypeName [trait1, trait2]:`
  const typeDefMatch = trimmed.match(/^(\w+)\s*\[([^\]]+)\]\s*:/);
  if (!typeDefMatch) return null;
  const bracketContent = typeDefMatch[2];
  const bracketStart = trimOffset + trimmed.indexOf('[') + 1;

  // Parse individual trait names
  const traits = bracketContent.split(',').map((t) => t.trim());
  let offset = bracketStart;
  for (const trait of traits) {
    // Account for leading whitespace in `[trait1, trait2]`
    const traitIdx = bracketContent.indexOf(trait, offset - bracketStart);
    const traitStart = bracketStart + traitIdx;
    const traitEnd = traitStart + trait.length;
    if (col >= traitStart && col < traitEnd) {
      return { kind: 'trait', name: trait };
    }
    offset = traitEnd;
  }
  return null;
}

// --- Type name detection ---

function findTypeName(trimmed: string, col: number, trimOffset: number): Reference | null {
  // 1. Type definition line: `TypeName:` or `TypeName [trait1, trait2]:`
  const defMatch = trimmed.match(/^([A-Z]\w*)(?:\s*\[([^\]]+)\])?\s*:/);
  if (defMatch) {
    const name = defMatch[1];
    const nameStart = trimOffset;
    const nameEnd = trimOffset + name.length;
    if (col >= nameStart && col < nameEnd) {
      const traits = defMatch[2] ? defMatch[2].split(',').map((t) => t.trim()) : undefined;
      return { kind: 'type-name', name, traits };
    }
  }

  // 2. Entity declaration line: `@entity_id: TypeName` or `@entity_id: TypeName { ... }`
  const entityDeclMatch = trimmed.match(/^@?\w+\s*:\s*([A-Z]\w*)/);
  if (entityDeclMatch) {
    const name = entityDeclMatch[1];
    const nameIdx = trimmed.indexOf(name, trimmed.indexOf(':'));
    const nameStart = trimOffset + nameIdx;
    const nameEnd = nameStart + name.length;
    if (col >= nameStart && col < nameEnd) {
      return { kind: 'type-name', name };
    }
  }

  return null;
}

// --- Hidden visibility prefix detection (F2) ---

function findVisibilityPrefix(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Property line starts with ~: `~prop: type`
  if (!trimmed.startsWith('~')) return null;
  const tildeCol = trimOffset;
  if (col === tildeCol) {
    return { kind: 'visibility-prefix', visibility: 'hidden' };
  }
  return null;
}

// --- Exit direction/destination detection ---

function findExitReference(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Exit line pattern: `direction: Destination Name`
  // Direction is indented, lowercase word; destination follows the colon.
  const match = trimmed.match(/^([a-z]+)\s*:\s*(.+)$/);
  if (!match) return null;

  const direction = match[1];
  const destination = match[2].trim();

  // Direction span
  const dirStart = trimOffset;
  const dirEnd = trimOffset + direction.length;
  if (col >= dirStart && col < dirEnd) {
    return { kind: 'exit-direction', direction };
  }

  // Destination span
  const destStart = trimOffset + match[0].indexOf(destination);
  const destEnd = destStart + destination.length;
  if (col >= destStart && col <= destEnd) {
    return { kind: 'exit-destination', destinationName: destination };
  }

  return null;
}
