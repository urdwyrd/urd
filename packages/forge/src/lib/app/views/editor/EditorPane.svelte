<script lang="ts">
  /**
   * EditorPane — CodeMirror 6 wrapper component.
   *
   * Owns the EditorView lifecycle, ResizeObserver, compartments for
   * theme/tabSize/lineNumbers/wordWrap/readOnly. Exposes methods for
   * external content updates, cursor positioning, and diagnostics.
   */

  import { onMount, onDestroy } from 'svelte';
  import { EditorView, lineNumbers, keymap } from '@codemirror/view';
  import { EditorState, Compartment, Annotation, type Extension } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { bracketMatching } from '@codemirror/language';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { lintGutter } from '@codemirror/lint';
  import { urdLanguage } from './urd-language';
  import { gloamingEditorTheme, parchmentEditorTheme, urdHighlight } from './urd-theme';
  import { setEditorDiagnostics, clearEditorDiagnostics } from './urd-diagnostics';
  import type { Diagnostic } from '$lib/app/compiler/types';

  interface Props {
    readOnly?: boolean;
    theme?: 'gloaming' | 'parchment';
    tabSize?: number;
    showLineNumbers?: boolean;
    wordWrap?: boolean;
    extraExtensions?: Extension[];
    onContentChange?: (content: string) => void;
    onCursorChange?: (line: number, col: number) => void;
  }

  let {
    readOnly = false,
    theme = 'gloaming',
    tabSize = 2,
    showLineNumbers = true,
    wordWrap = false,
    extraExtensions = [],
    onContentChange,
    onCursorChange,
  }: Props = $props();

  /** Annotation used to mark changes that came from external sources (not the user typing). */
  const externalChange = Annotation.define<boolean>();

  let containerEl: HTMLDivElement | undefined = $state();
  let view: EditorView | undefined;

  // Compartments for dynamic reconfiguration
  const themeCompartment = new Compartment();
  const tabSizeCompartment = new Compartment();
  const lineNumbersCompartment = new Compartment();
  const wordWrapCompartment = new Compartment();
  const readOnlyCompartment = new Compartment();
  const extraCompartment = new Compartment();

  function getThemeExtension(themeName: string): Extension {
    return themeName === 'parchment' ? parchmentEditorTheme : gloamingEditorTheme;
  }

  onMount(() => {
    if (!containerEl) return;

    const state = EditorState.create({
      doc: '',
      extensions: [
        // Core
        history(),
        bracketMatching(),
        closeBrackets(),
        highlightSelectionMatches(),
        lintGutter(),
        // Language
        urdLanguage,
        urdHighlight,
        // Compartments
        themeCompartment.of(getThemeExtension(theme)),
        tabSizeCompartment.of(EditorState.tabSize.of(tabSize)),
        lineNumbersCompartment.of(showLineNumbers ? lineNumbers() : []),
        wordWrapCompartment.of(wordWrap ? EditorView.lineWrapping : []),
        readOnlyCompartment.of(EditorState.readOnly.of(readOnly)),
        extraCompartment.of(extraExtensions),
        // Keymaps
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...historyKeymap,
          ...searchKeymap,
        ]),
        // Change listener
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            // Only fire for user-initiated changes
            const isExternal = update.transactions.some(
              (tr) => tr.annotation(externalChange)
            );
            if (!isExternal && onContentChange) {
              onContentChange(update.state.doc.toString());
            }
          }
          if (update.selectionSet && onCursorChange) {
            const pos = update.state.selection.main.head;
            const line = update.state.doc.lineAt(pos);
            onCursorChange(line.number, pos - line.from + 1);
          }
        }),
      ],
    });

    view = new EditorView({ state, parent: containerEl });

    // ResizeObserver for responsive layout
    const resizeObserver = new ResizeObserver(() => {
      view?.requestMeasure();
    });
    resizeObserver.observe(containerEl);

    return () => {
      resizeObserver.disconnect();
    };
  });

  onDestroy(() => {
    view?.destroy();
    view = undefined;
  });

  // Reactive compartment updates
  $effect(() => {
    view?.dispatch({
      effects: themeCompartment.reconfigure(getThemeExtension(theme)),
    });
  });

  $effect(() => {
    view?.dispatch({
      effects: tabSizeCompartment.reconfigure(EditorState.tabSize.of(tabSize)),
    });
  });

  $effect(() => {
    view?.dispatch({
      effects: lineNumbersCompartment.reconfigure(showLineNumbers ? lineNumbers() : []),
    });
  });

  $effect(() => {
    view?.dispatch({
      effects: wordWrapCompartment.reconfigure(wordWrap ? EditorView.lineWrapping : []),
    });
  });

  $effect(() => {
    view?.dispatch({
      effects: readOnlyCompartment.reconfigure(EditorState.readOnly.of(readOnly)),
    });
  });

  $effect(() => {
    view?.dispatch({
      effects: extraCompartment.reconfigure(extraExtensions),
    });
  });

  // --- Public API ---

  /** Replace the entire document content (marks as external change). */
  export function setContent(content: string): void {
    if (!view) return;
    const currentContent = view.state.doc.toString();
    if (currentContent === content) return;

    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: content },
      annotations: externalChange.of(true),
    });
  }

  /** Get the current document content. */
  export function getContent(): string {
    return view?.state.doc.toString() ?? '';
  }

  /** Scroll to a specific line number and optionally set cursor. */
  export function scrollToLine(line: number, col?: number): void {
    if (!view) return;
    const lineCount = view.state.doc.lines;
    const clampedLine = Math.max(1, Math.min(line, lineCount));
    const lineInfo = view.state.doc.line(clampedLine);
    const pos = lineInfo.from + Math.max(0, (col ?? 1) - 1);

    view.dispatch({
      selection: { anchor: pos },
      scrollIntoView: true,
    });
    view.focus();
  }

  /** Get the current cursor position. */
  export function getCursorPosition(): { line: number; col: number } {
    if (!view) return { line: 1, col: 1 };
    const pos = view.state.selection.main.head;
    const line = view.state.doc.lineAt(pos);
    return { line: line.number, col: pos - line.from + 1 };
  }

  /** Get the scroll position. */
  export function getScrollTop(): number {
    return view?.scrollDOM.scrollTop ?? 0;
  }

  /** Set the scroll position. */
  export function setScrollTop(top: number): void {
    if (view) {
      view.scrollDOM.scrollTop = top;
    }
  }

  /** Get the underlying EditorView (for extensions that need direct access). */
  export function getView(): EditorView | undefined {
    return view;
  }

  /** Push diagnostics into the editor view. */
  export function setDiagnostics(diagnostics: Diagnostic[]): void {
    if (view) {
      setEditorDiagnostics(view, diagnostics);
    }
  }

  /** Clear all diagnostics from the editor view. */
  export function clearDiagnostics(): void {
    if (view) {
      clearEditorDiagnostics(view);
    }
  }

  /** Focus the editor. */
  export function focus(): void {
    view?.focus();
  }
</script>

<div class="forge-editor-pane" bind:this={containerEl}></div>

<style>
  .forge-editor-pane {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-editor-pane :global(.cm-editor) {
    height: 100%;
  }

  .forge-editor-pane :global(.cm-scroller) {
    overflow: auto;
  }
</style>
