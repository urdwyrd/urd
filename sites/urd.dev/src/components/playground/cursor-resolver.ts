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
  | { kind: 'type-constructor'; name: string }
  | { kind: 'exit-direction'; direction: string }
  | { kind: 'exit-destination'; destinationName: string };

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
  // 1. Section label: == name
  const trimmed = line.trimStart();
  const trimOffset = line.length - trimmed.length;
  if (trimmed.startsWith('== ') || trimmed.startsWith('==')) {
    const prefixLen = trimmed.startsWith('== ') ? 3 : 2;
    const name = trimmed.slice(prefixLen).trim();
    if (name.length > 0) {
      return { kind: 'section-label', name };
    }
  }

  // 2. Location heading: # (single # only, not ##)
  if (trimmed.startsWith('# ') && !trimmed.startsWith('## ')) {
    const name = trimmed.slice(2).trim();
    if (name.length > 0) {
      return { kind: 'location-heading', name };
    }
  }

  // 3. Frontmatter delimiter: ---
  if (/^---\s*$/.test(trimmed) && col >= trimOffset && col <= trimOffset + 3) {
    return { kind: 'keyword', token: '---' };
  }

  // 4. Section jump: -> target (check for -> END / -> RETURN first)
  const arrowIdx = line.indexOf('->');
  if (arrowIdx >= 0 && col >= arrowIdx) {
    const afterArrow = line.slice(arrowIdx + 2).trim();
    if (afterArrow === 'END' || afterArrow === 'RETURN') {
      return { kind: 'keyword', token: `-> ${afterArrow}` };
    }
    // Regular section jump
    const arrowResult = findSectionJump(line, col);
    if (arrowResult) return arrowResult;
    // Cursor on the -> itself but no target resolved
    if (col >= arrowIdx && col < arrowIdx + 2) {
      return { kind: 'keyword', token: '->' };
    }
  }

  // 5. Line-start structural markers: + * ? >
  const keywordResult = findLineStartKeyword(trimmed, col, trimOffset);
  if (keywordResult) return keywordResult;

  // 6. Frontmatter-aware references
  if (context?.inFrontmatter) {
    const fmKeyResult = findFrontmatterKey(trimmed, col, trimOffset);
    if (fmKeyResult) return fmKeyResult;

    const typeConResult = findTypeConstructor(trimmed, col, trimOffset, context);
    if (typeConResult) return typeConResult;
  }

  // 7. Exit direction/destination: `direction: Destination Name` (body only)
  if (!context?.inFrontmatter) {
    const exitResult = findExitReference(trimmed, col, trimOffset);
    if (exitResult) return exitResult;
  }

  // 8. Entity / entity.property reference
  const entityResult = findEntityReference(line, col);
  if (entityResult) return entityResult;

  // 9. Type.property reference (uppercase start before dot)
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
  let sawTypesKey = false;

  for (let i = 1; i <= Math.min(lineNumber, doc.lines); i++) {
    const text = doc.line(i).text.trimEnd();
    if (/^---\s*$/.test(text)) {
      delimCount++;
      if (delimCount >= 2) break;
      continue;
    }
    if (delimCount === 1) {
      // Inside frontmatter — track type block
      if (/^\w/.test(text) && text.includes(':')) {
        sawTypesKey = /^types\s*:/.test(text);
      }
      // If line is indented and we're past `types:`, we may be in a type block
      if (i === lineNumber && sawTypesKey && /^\s{2,}/.test(doc.line(i).text)) {
        inTypeBlock = true;
      }
    }
  }

  const inFrontmatter = delimCount === 1 && lineNumber > 1;
  return { inFrontmatter, inTypeBlock };
}

// --- Line-start keyword detection ---

function findLineStartKeyword(trimmed: string, col: number, trimOffset: number): Reference | null {
  // Only match markers at the start of the (trimmed) line
  const markers: [RegExp, string][] = [
    [/^\+\s/, '+'],
    [/^\*\s/, '*'],
    [/^\?\s/, '?'],
    [/^>\s/, '>'],
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
  // Property line: `  property_name: type_constructor` or `    property_name: type_constructor(...)`
  const match = trimmed.match(/^(\w+)\s*:\s*(\w+)/);
  if (!match) return null;
  const typeCon = match[2];
  if (!TYPE_CONSTRUCTORS.has(typeCon)) return null;
  // Find the column range of the constructor in the original line
  const typeConStart = trimOffset + match.index! + match[0].indexOf(typeCon);
  const typeConEnd = typeConStart + typeCon.length;
  if (col >= typeConStart && col < typeConEnd) {
    return { kind: 'type-constructor', name: typeCon };
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
