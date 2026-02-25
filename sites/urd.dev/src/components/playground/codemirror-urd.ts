/**
 * CodeMirror 6 language mode for Urd Schema Markdown + Gloaming/Parchment editor themes.
 *
 * Line-oriented pattern matching — not a full grammar parse.
 * Sufficient for visual differentiation in the playground.
 */

import { StreamLanguage, type StringStream, type StreamParser } from '@codemirror/language';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { EditorView } from '@codemirror/view';
import { tags } from '@lezer/highlight';

// --- Schema Markdown StreamLanguage mode ---

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
      // Skip leading whitespace
      if (stream.eatSpace()) return null;

      // Strings in frontmatter
      if (stream.match(/"[^"]*"/)) return 'string';

      // Numbers in frontmatter
      if ((stream.pos === 0 || !/\w/.test(stream.string[stream.pos - 1])) &&
          stream.match(/\b\d+\b/)) return 'number';

      // Keywords (guard: previous char must not be a word char)
      if ((stream.pos === 0 || !/\w/.test(stream.string[stream.pos - 1])) &&
          stream.match(/\b(true|false|import)\b/)) return 'keyword';

      // Entity references in frontmatter
      if (stream.match(/@[\w.-]+/)) return 'variableName';

      // Frontmatter keys (word followed by colon)
      if (stream.match(/[\w-]+(?=\s*:)/)) return 'propertyName';

      // Everything else in frontmatter
      stream.next();
      return null;
    }

    // Start of line patterns
    if (stream.sol()) {
      // Consume leading whitespace
      const indent = stream.eatSpace();

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
      if (stream.match(/^\?\s/)) {
        tokeniseInline(stream);
        return 'keyword';
      }

      // Effects
      if (stream.match(/^>\s/)) {
        tokeniseInline(stream);
        return 'keyword';
      }

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
      if (stream.match(/@[\w.-]+:/)) {
        return 'variableName';
      }

      // Presence markers: [@word]
      if (stream.match(/\[@[\w.-]+\]/)) {
        return 'variableName';
      }
    }

    // Inline patterns (mid-line)

    // Jump arrows
    if (stream.match(/->/)) return 'keyword';

    // Entity references
    if (stream.match(/@[\w.-]+/)) return 'variableName';

    // Numbers
    if ((stream.pos === 0 || !/\w/.test(stream.string[stream.pos - 1])) &&
        stream.match(/\b\d+\b/)) return 'number';

    // Keywords (guard: previous char must not be a word char)
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

/** Skip to end of line — used after condition/effect markers. */
function tokeniseInline(_stream: StringStream): void {
  // The marker itself is already consumed and styled.
  // The rest of the line remains default-styled for now.
}

export const urdLanguage = StreamLanguage.define(urdParser);

// --- Gloaming Editor Theme ---

export const gloamingTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--deep)',
    color: 'var(--text)',
    fontFamily: 'var(--mono)',
    fontSize: '13px',
    lineHeight: '1.6',
  },
  '.cm-content': {
    caretColor: 'var(--gold)',
    fontFamily: 'var(--mono)',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeftColor: 'var(--gold)',
    borderLeftWidth: '2px',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--surface) !important',
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(26, 27, 37, 0.5)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--raise)',
    color: 'var(--faint)',
    borderRight: '1px solid var(--border)',
    fontFamily: 'var(--mono)',
    fontSize: '12px',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--surface)',
    color: 'var(--dim)',
  },
  '.cm-lineNumbers .cm-gutterElement': {
    padding: '0 8px 0 4px',
  },
  '.cm-matchingBracket': {
    backgroundColor: 'rgba(218, 184, 96, 0.2)',
    outline: '1px solid rgba(218, 184, 96, 0.4)',
  },
  '.cm-searchMatch': {
    backgroundColor: 'rgba(218, 184, 96, 0.15)',
    outline: '1px solid rgba(218, 184, 96, 0.3)',
  },
  '.cm-searchMatch.cm-searchMatch-selected': {
    backgroundColor: 'rgba(218, 184, 96, 0.3)',
  },
  '.cm-placeholder': {
    color: 'var(--faint)',
    fontStyle: 'italic',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--surface)',
    border: '1px solid var(--border)',
    color: 'var(--text)',
  },
  '.cm-panels': {
    backgroundColor: 'var(--raise)',
    color: 'var(--text)',
  },
  '.cm-panels.cm-panels-top': {
    borderBottom: '1px solid var(--border)',
  },
  '.cm-foldPlaceholder': {
    backgroundColor: 'var(--surface)',
    border: '1px solid var(--border)',
    color: 'var(--faint)',
  },
  // Lint decorations
  '.cm-lintRange-error': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%23cc8888' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-lintRange-warning': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%23e8a060' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-lintRange-info': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%236a9acc' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-lint-marker-error': {
    content: '"◆"',
  },
  '.cm-lint-marker-warning': {
    content: '"▸"',
  },
  // Definition link (Ctrl/Cmd hover underline)
  '.cm-definition-link': {
    textDecoration: 'underline',
    textDecorationColor: 'var(--blue-light)',
    cursor: 'pointer',
  },
  // Autocomplete tooltip
  '.cm-tooltip-autocomplete': {
    backgroundColor: 'var(--surface)',
    border: '1px solid var(--border)',
  },
  '.cm-tooltip-autocomplete ul li[aria-selected]': {
    backgroundColor: 'var(--raise)',
    color: 'var(--text)',
  },
  '.cm-completionLabel': {
    color: 'var(--text)',
  },
  '.cm-completionDetail': {
    color: 'var(--faint)',
    fontStyle: 'italic',
  },
  // Hover tooltip
  '.urd-hover-tooltip': {
    fontFamily: 'var(--mono)',
    fontSize: '12px',
    lineHeight: '1.5',
    padding: '6px 10px',
    maxWidth: '400px',
  },
  '.urd-tt-dim': {
    color: 'var(--faint)',
    fontSize: '11px',
  },
  '.urd-tt-warn': {
    color: 'var(--amber)',
    fontSize: '11px',
  },
}, { dark: true });

// --- Parchment Editor Theme ---

/** Shared UI styles — identical between themes (use CSS variables). */
const sharedThemeStyles = {
  '&': {
    backgroundColor: 'var(--deep)',
    color: 'var(--text)',
    fontFamily: 'var(--mono)',
    fontSize: '13px',
    lineHeight: '1.6',
  },
  '.cm-content': {
    caretColor: 'var(--gold)',
    fontFamily: 'var(--mono)',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeftColor: 'var(--gold)',
    borderLeftWidth: '2px',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--surface) !important',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--raise)',
    color: 'var(--faint)',
    borderRight: '1px solid var(--border)',
    fontFamily: 'var(--mono)',
    fontSize: '12px',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--surface)',
    color: 'var(--dim)',
  },
  '.cm-lineNumbers .cm-gutterElement': {
    padding: '0 8px 0 4px',
  },
  '.cm-placeholder': {
    color: 'var(--faint)',
    fontStyle: 'italic',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--surface)',
    border: '1px solid var(--border)',
    color: 'var(--text)',
  },
  '.cm-panels': {
    backgroundColor: 'var(--raise)',
    color: 'var(--text)',
  },
  '.cm-panels.cm-panels-top': {
    borderBottom: '1px solid var(--border)',
  },
  '.cm-foldPlaceholder': {
    backgroundColor: 'var(--surface)',
    border: '1px solid var(--border)',
    color: 'var(--faint)',
  },
  '.cm-lint-marker-error': {
    content: '"◆"',
  },
  '.cm-lint-marker-warning': {
    content: '"▸"',
  },
  '.cm-definition-link': {
    textDecoration: 'underline',
    textDecorationColor: 'var(--gold-dim)',
    cursor: 'pointer',
  },
  '.cm-tooltip-autocomplete': {
    backgroundColor: 'var(--surface)',
    border: '1px solid var(--border)',
  },
  '.cm-tooltip-autocomplete ul li[aria-selected]': {
    backgroundColor: 'var(--raise)',
    color: 'var(--text)',
  },
  '.cm-completionLabel': {
    color: 'var(--text)',
  },
  '.cm-completionDetail': {
    color: 'var(--faint)',
    fontStyle: 'italic',
  },
  '.urd-hover-tooltip': {
    fontFamily: 'var(--mono)',
    fontSize: '12px',
    lineHeight: '1.5',
    padding: '6px 10px',
    maxWidth: '400px',
  },
  '.urd-tt-dim': {
    color: 'var(--faint)',
    fontSize: '11px',
  },
  '.urd-tt-warn': {
    color: 'var(--amber)',
    fontSize: '11px',
  },
};

export const parchmentTheme = EditorView.theme({
  ...sharedThemeStyles,
  '.cm-activeLine': {
    backgroundColor: 'rgba(45, 40, 24, 0.06)',
  },
  '.cm-matchingBracket': {
    backgroundColor: 'rgba(138, 109, 24, 0.15)',
    outline: '1px solid rgba(138, 109, 24, 0.3)',
  },
  '.cm-searchMatch': {
    backgroundColor: 'rgba(138, 109, 24, 0.12)',
    outline: '1px solid rgba(138, 109, 24, 0.2)',
  },
  '.cm-searchMatch.cm-searchMatch-selected': {
    backgroundColor: 'rgba(138, 109, 24, 0.25)',
  },
  '.cm-lintRange-error': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%23a05050' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-lintRange-warning': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%23a05a1a' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-lintRange-info': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%233a6a98' fill='none' stroke-width='1.1'/></svg>")`,
  },
}, { dark: false });

// --- Syntax Highlighting (shared — uses CSS variables, adapts per theme) ---

export const urdHighlight = syntaxHighlighting(HighlightStyle.define([
  // Frontmatter delimiters + comments
  { tag: tags.meta, color: 'var(--faint)' },
  { tag: tags.comment, color: 'var(--faint)', fontStyle: 'italic' },

  // Frontmatter keys
  { tag: tags.propertyName, color: 'var(--amber)' },

  // Headings + section labels
  { tag: tags.heading, color: 'var(--gold)', fontWeight: '600' },

  // Entity references + dialogue attribution
  { tag: tags.variableName, color: 'var(--blue-light)' },

  // Conditions (? lines), effects (> lines), jumps (->), keywords
  { tag: tags.keyword, color: 'var(--green)' },

  // Choices (+ and *)
  { tag: tags.operator, color: 'var(--purple)' },

  // Strings
  { tag: tags.string, color: 'var(--green-light)' },

  // Numbers
  { tag: tags.number, color: 'var(--amber-light)' },
]));
