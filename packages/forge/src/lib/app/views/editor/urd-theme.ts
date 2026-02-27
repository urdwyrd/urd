/**
 * CodeMirror 6 editor themes for Forge — Gloaming (dark) and Parchment (light).
 *
 * Adapted from the playground's codemirror-urd.ts theme section.
 * Uses --forge-syntax-* CSS custom properties defined in tokens.css.
 */

import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags } from '@lezer/highlight';

// --- Shared UI styles (both themes use CSS variables for adaptation) ---

const sharedStyles: Record<string, Record<string, string>> = {
  '&': {
    fontFamily: 'var(--forge-font-family-mono)',
    fontSize: 'var(--forge-editor-font-size, 13px)',
    lineHeight: '1.6',
  },
  '.cm-content': {
    fontFamily: 'var(--forge-font-family-mono)',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeftWidth: '2px',
  },
  '.cm-gutters': {
    fontFamily: 'var(--forge-font-family-mono)',
    fontSize: '12px',
  },
  '.cm-lineNumbers .cm-gutterElement': {
    padding: '0 8px 0 4px',
  },
  '.cm-placeholder': {
    fontStyle: 'italic',
  },
  '.cm-foldPlaceholder': {
    border: '1px solid var(--forge-border-zone)',
  },
  // Lint markers
  '.cm-lint-marker-error': {
    content: '"◆"',
  },
  '.cm-lint-marker-warning': {
    content: '"▸"',
  },
  // Definition link (Ctrl/Cmd hover underline)
  '.cm-definition-link': {
    textDecoration: 'underline',
    cursor: 'pointer',
  },
  // Hover tooltip
  '.urd-hover-tooltip': {
    fontFamily: 'var(--forge-font-family-mono)',
    fontSize: '12px',
    lineHeight: '1.5',
    padding: '6px 10px',
    maxWidth: '600px',
  },
  '.urd-tt-dim': {
    fontSize: '11px',
  },
  '.urd-tt-warn': {
    fontSize: '11px',
  },
  '.urd-tt-error': {
    fontSize: '11px',
  },
};

// --- Gloaming (dark) theme ---

export const gloamingEditorTheme = EditorView.theme({
  ...sharedStyles,
  '&': {
    ...sharedStyles['&'],
    backgroundColor: 'var(--forge-bg-zone-viewport)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-content': {
    ...sharedStyles['.cm-content'],
    caretColor: 'var(--forge-syntax-heading)',
  },
  '.cm-cursor, .cm-dropCursor': {
    ...sharedStyles['.cm-cursor, .cm-dropCursor'],
    borderLeftColor: 'var(--forge-syntax-heading)',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--forge-accent-selection) !important',
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(26, 27, 37, 0.5)',
  },
  '.cm-gutters': {
    ...sharedStyles['.cm-gutters'],
    backgroundColor: 'var(--forge-bg-secondary)',
    color: 'var(--forge-text-muted)',
    borderRight: '1px solid var(--forge-border-zone)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--forge-bg-tertiary)',
    color: 'var(--forge-text-secondary)',
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
    ...sharedStyles['.cm-placeholder'],
    color: 'var(--forge-text-muted)',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--forge-bg-secondary)',
    border: '1px solid var(--forge-border-zone)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-panels': {
    backgroundColor: 'var(--forge-bg-secondary)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-panels.cm-panels-top': {
    borderBottom: '1px solid var(--forge-border-zone)',
  },
  '.cm-foldPlaceholder': {
    ...sharedStyles['.cm-foldPlaceholder'],
    backgroundColor: 'var(--forge-bg-secondary)',
    color: 'var(--forge-text-muted)',
  },
  // Lint squigglies
  '.cm-lintRange-error': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%23cc8888' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-lintRange-warning': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%23e8a060' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-lintRange-info': {
    backgroundImage: `url("data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='m0 3 l2 -2 l1 0 l2 2 l1 0' stroke='%236a9acc' fill='none' stroke-width='1.1'/></svg>")`,
  },
  '.cm-definition-link': {
    ...sharedStyles['.cm-definition-link'],
    textDecorationColor: 'var(--forge-syntax-entity)',
  },
  // Autocomplete
  '.cm-tooltip-autocomplete': {
    backgroundColor: 'var(--forge-bg-secondary)',
    border: '1px solid var(--forge-border-zone)',
  },
  '.cm-tooltip-autocomplete ul li[aria-selected]': {
    backgroundColor: 'var(--forge-bg-tertiary)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-completionLabel': {
    color: 'var(--forge-text-primary)',
  },
  '.cm-completionDetail': {
    color: 'var(--forge-text-muted)',
    fontStyle: 'italic',
  },
  '.urd-tt-dim': {
    ...sharedStyles['.urd-tt-dim'],
    color: 'var(--forge-text-muted)',
  },
  '.urd-tt-warn': {
    ...sharedStyles['.urd-tt-warn'],
    color: 'var(--forge-status-warning)',
  },
  '.urd-tt-error': {
    ...sharedStyles['.urd-tt-error'],
    color: 'var(--forge-status-error)',
  },
}, { dark: true });

// --- Parchment (light) theme ---

export const parchmentEditorTheme = EditorView.theme({
  ...sharedStyles,
  '&': {
    ...sharedStyles['&'],
    backgroundColor: 'var(--forge-bg-zone-viewport)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-content': {
    ...sharedStyles['.cm-content'],
    caretColor: 'var(--forge-syntax-heading)',
  },
  '.cm-cursor, .cm-dropCursor': {
    ...sharedStyles['.cm-cursor, .cm-dropCursor'],
    borderLeftColor: 'var(--forge-syntax-heading)',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--forge-accent-selection) !important',
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(45, 40, 24, 0.06)',
  },
  '.cm-gutters': {
    ...sharedStyles['.cm-gutters'],
    backgroundColor: 'var(--forge-bg-secondary)',
    color: 'var(--forge-text-muted)',
    borderRight: '1px solid var(--forge-border-zone)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--forge-bg-tertiary)',
    color: 'var(--forge-text-secondary)',
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
  '.cm-placeholder': {
    ...sharedStyles['.cm-placeholder'],
    color: 'var(--forge-text-muted)',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--forge-bg-secondary)',
    border: '1px solid var(--forge-border-zone)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-panels': {
    backgroundColor: 'var(--forge-bg-secondary)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-panels.cm-panels-top': {
    borderBottom: '1px solid var(--forge-border-zone)',
  },
  '.cm-foldPlaceholder': {
    ...sharedStyles['.cm-foldPlaceholder'],
    backgroundColor: 'var(--forge-bg-secondary)',
    color: 'var(--forge-text-muted)',
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
  '.cm-definition-link': {
    ...sharedStyles['.cm-definition-link'],
    textDecorationColor: 'var(--forge-syntax-heading)',
  },
  '.cm-tooltip-autocomplete': {
    backgroundColor: 'var(--forge-bg-secondary)',
    border: '1px solid var(--forge-border-zone)',
  },
  '.cm-tooltip-autocomplete ul li[aria-selected]': {
    backgroundColor: 'var(--forge-bg-tertiary)',
    color: 'var(--forge-text-primary)',
  },
  '.cm-completionLabel': {
    color: 'var(--forge-text-primary)',
  },
  '.cm-completionDetail': {
    color: 'var(--forge-text-muted)',
    fontStyle: 'italic',
  },
  '.urd-tt-dim': {
    ...sharedStyles['.urd-tt-dim'],
    color: 'var(--forge-text-muted)',
  },
  '.urd-tt-warn': {
    ...sharedStyles['.urd-tt-warn'],
    color: 'var(--forge-status-warning)',
  },
  '.urd-tt-error': {
    ...sharedStyles['.urd-tt-error'],
    color: 'var(--forge-status-error)',
  },
}, { dark: false });

// --- Syntax highlighting (shared — uses CSS custom properties, adapts per theme) ---

export const urdHighlight = syntaxHighlighting(HighlightStyle.define([
  // Frontmatter delimiters + meta
  { tag: tags.meta, color: 'var(--forge-syntax-meta)' },
  // Comments
  { tag: tags.comment, color: 'var(--forge-syntax-comment)', fontStyle: 'italic' },
  // Frontmatter keys
  { tag: tags.propertyName, color: 'var(--forge-syntax-property)' },
  // Headings + section labels
  { tag: tags.heading, color: 'var(--forge-syntax-heading)', fontWeight: '600' },
  // Entity references + dialogue attribution
  { tag: tags.variableName, color: 'var(--forge-syntax-entity)' },
  // Conditions, effects, jumps, keywords
  { tag: tags.keyword, color: 'var(--forge-syntax-keyword)' },
  // Choices (+ and *)
  { tag: tags.operator, color: 'var(--forge-syntax-operator)' },
  // Strings
  { tag: tags.string, color: 'var(--forge-syntax-string)' },
  // Numbers
  { tag: tags.number, color: 'var(--forge-syntax-number)' },
]));
