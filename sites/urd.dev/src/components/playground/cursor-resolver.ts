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
  | { kind: 'location-heading'; name: string };

/**
 * Identify the Urd reference at the given column in a line of text.
 * Column is 0-indexed character offset from line start.
 */
export function identifyReference(line: string, col: number): Reference | null {
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

  // 3. Section jump: -> target
  const arrowResult = findSectionJump(line, col);
  if (arrowResult) return arrowResult;

  // 4. Entity / entity.property reference
  const entityResult = findEntityReference(line, col);
  if (entityResult) return entityResult;

  // 5. Type.property reference (uppercase start before dot)
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
