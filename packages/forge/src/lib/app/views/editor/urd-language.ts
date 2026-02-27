/**
 * Urd Schema Markdown language mode for CodeMirror 6.
 *
 * Line-oriented pattern matching via StreamLanguage — not a full grammar parse.
 * Ported from the playground's codemirror-urd.ts (StreamParser section).
 */

import { StreamLanguage, type StringStream, type StreamParser } from '@codemirror/language';

interface UrdState {
  inFrontmatter: boolean;
  frontmatterOpen: boolean;
}

const urdParser: StreamParser<UrdState> = {
  startState(): UrdState {
    return { inFrontmatter: false, frontmatterOpen: false };
  },

  token(stream: StringStream, state: UrdState): string | null {
    // Frontmatter delimiters
    if (stream.sol() && stream.match(/^---\s*$/)) {
      if (!state.frontmatterOpen) {
        state.inFrontmatter = true;
        state.frontmatterOpen = true;
      } else {
        state.inFrontmatter = false;
      }
      return 'meta';
    }

    // Inside frontmatter
    if (state.inFrontmatter) {
      if (stream.eatSpace()) return null;

      // Strings
      if (stream.match(/"[^"]*"/)) return 'string';

      // Numbers
      if ((stream.pos === 0 || !/\w/.test(stream.string[stream.pos - 1])) &&
          stream.match(/\b\d+\b/)) return 'number';

      // Keywords
      if ((stream.pos === 0 || !/\w/.test(stream.string[stream.pos - 1])) &&
          stream.match(/\b(true|false|import)\b/)) return 'keyword';

      // Entity references in frontmatter
      if (stream.match(/@[\w.-]+/)) return 'variableName';

      // Frontmatter keys (word followed by colon)
      if (stream.match(/[\w-]+(?=\s*:)/)) return 'propertyName';

      // Default
      stream.next();
      return null;
    }

    // Start of line patterns
    if (stream.sol()) {
      stream.eatSpace();

      // Comments
      if (stream.match(/\/\/.*/)) return 'comment';

      // Headings
      if (stream.match(/^#{1,3}\s/)) {
        stream.skipToEnd();
        return 'heading';
      }

      // Section labels
      if (stream.match(/^==\s/)) {
        stream.skipToEnd();
        return 'heading';
      }

      // Conditions
      if (stream.match(/^\?\s/)) return 'keyword';

      // Effects
      if (stream.match(/^>\s/)) return 'keyword';

      // Sticky choices
      if (stream.match(/^\+\s/)) {
        stream.skipToEnd();
        return 'operator';
      }

      // One-shot choices
      if (stream.match(/^\*\s/)) {
        stream.skipToEnd();
        return 'operator';
      }

      // Blocked messages
      if (stream.match(/^!\s/)) {
        stream.skipToEnd();
        return 'keyword';
      }

      // Dialogue attribution: @word: at line start (with optional indent)
      if (stream.match(/@[\w.-]+:/)) return 'variableName';

      // Presence markers: [@word]
      if (stream.match(/\[@[\w.-]+\]/)) return 'variableName';
    }

    // Inline patterns (mid-line)

    // Jump arrows
    if (stream.match(/->/)) return 'keyword';

    // Entity references
    if (stream.match(/@[\w.-]+/)) return 'variableName';

    // Numbers
    if ((stream.pos === 0 || !/\w/.test(stream.string[stream.pos - 1])) &&
        stream.match(/\b\d+\b/)) return 'number';

    // Keywords
    if ((stream.pos === 0 || !/\w/.test(stream.string[stream.pos - 1])) &&
        stream.match(/\b(true|false|in|not in|import)\b/)) return 'keyword';

    // Strings
    if (stream.match(/"[^"]*"/)) return 'string';

    // Comments anywhere
    if (stream.match(/\/\/.*/)) return 'comment';

    // Default: advance one character
    stream.next();
    return null;
  },
};

export const urdLanguage = StreamLanguage.define(urdParser);
