<script lang="ts">
  import { onMount } from 'svelte';
  import OutputPane from './OutputPane.svelte';
  import {
    initCompiler,
    compileSource,
    parseOnly,
    isReady,
    byteColToCharCol,
    type CompileResult,
  } from './compiler-bridge';

  // --- Constants ---

  const PARSE_DEBOUNCE_MS = 50;
  const COMPILE_DEBOUNCE_MS = 300;

  const STARTER_EXAMPLE = `---
world:
  name: The Locked Garden
  start: gatehouse

types:
  Character [interactable]:
    mood: enum(wary, neutral, friendly) = wary
    trust: integer = 0
    ~role: string
  Item [portable]:
    name: string
    used: bool = false
  Lock [interactable]:
    locked: bool = true

entities:
  @warden: Character { role: "Gatekeeper", mood: "neutral" }
  @ghost: Character { role: "The Forgotten", trust: 3 }
  @iron_key: Item { name: "Iron Key" }
  @journal: Item { name: "Warden's Journal" }
  @garden_gate: Lock
---

# Gatehouse

A stone archway choked with ivy. Lantern light flickers.

[@warden, @iron_key]

-> garden: The Walled Garden
  ? @garden_gate.locked == false
  ! The gate is sealed with old iron.

== greet

@warden: Nobody passes without reason.

+ State your purpose
  @warden: Hmm. Lots of people say that.
  > @warden.trust + 1
  -> greet

* Offer the journal
  ? @journal in player
  @warden: Where did you find this? This changes things.
  > @warden.trust + 5
  > @warden.mood = friendly

* Ask about the garden
  ? @warden.trust >= 3
  @warden: The garden remembers what we buried there.
  > @garden_gate.locked = false

* Force the gate -> @garden_gate
  ? @warden.mood != friendly
  @warden: I wouldn't try that.
  > @warden.trust - 2

# The Walled Garden

Overgrown paths wind between crumbling statues.

[@ghost, @journal]

-> north: Gatehouse

== explore

@ghost: You shouldn't have come here.

? any:
  @ghost.trust >= 5
  @journal in player

* Listen to the ghost
  @ghost: They locked this place to forget. But I remember everything.
  > @ghost.trust + 2

  * Press for the truth
    ? @ghost.trust >= 5
    @ghost: The key opens more than gates. Take it to the fountain.
    -> revelation

* Take the journal
  ? @journal in here
  @ghost: That belongs to the living. Perhaps it still matters.

* Leave quietly
  @ghost: The garden never forgets.

== revelation

@ghost: Now you know.

> destroy @iron_key
`;

  // --- State ---

  let compilerReady = $state(false);
  let loadError: string | null = $state(null);
  let compileResult: CompileResult | null = $state(null);
  let compileTimeMs = $state(0);
  let parseValid = $state(true);
  let splitPercent = $state(50);
  let mobileTab: 'editor' | 'output' = $state('editor');

  // DOM refs
  let editorContainer: HTMLDivElement | undefined = $state();
  let dividerEl: HTMLDivElement | undefined = $state();
  let playgroundEl: HTMLDivElement | undefined = $state();

  // CodeMirror instance (stored outside $state to avoid reactivity overhead)
  let editorView: import('@codemirror/view').EditorView | undefined;

  // Debounce timers
  let parseTimer: ReturnType<typeof setTimeout> | undefined;
  let compileTimer: ReturnType<typeof setTimeout> | undefined;

  // Sequence counter to discard stale results
  let compileSeq = 0;

  // --- Lifecycle ---

  onMount(() => {
    let destroyed = false;

    async function setup() {
      // 1. Mount CodeMirror
      const { EditorView, keymap, placeholder, lineNumbers, highlightActiveLine, drawSelection } = await import('@codemirror/view');
      const { EditorState } = await import('@codemirror/state');
      const { defaultKeymap, history, historyKeymap, indentWithTab } = await import('@codemirror/commands');
      const { bracketMatching, indentOnInput } = await import('@codemirror/language');
      const { lintGutter } = await import('@codemirror/lint');
      const { urdLanguage, gloamingTheme, gloamingHighlight } = await import('./codemirror-urd');

      if (destroyed) return;

      editorView = new EditorView({
        parent: editorContainer!,
        state: EditorState.create({
          doc: STARTER_EXAMPLE,
          extensions: [
            lineNumbers(),
            highlightActiveLine(),
            drawSelection(),
            history(),
            bracketMatching(),
            indentOnInput(),
            lintGutter(),
            EditorView.lineWrapping,
            placeholder('Type Schema Markdown here…'),
            keymap.of([
              ...defaultKeymap,
              ...historyKeymap,
              indentWithTab,
            ]),
            EditorState.tabSize.of(2),
            urdLanguage,
            gloamingTheme,
            gloamingHighlight,
            EditorView.updateListener.of((update) => {
              if (update.docChanged) {
                onSourceChange(update.state.doc.toString());
              }
            }),
          ],
        }),
      });

      // 2. Load WASM compiler
      try {
        await initCompiler();
        if (destroyed) return;
        compilerReady = true;
        // Compile the starter example
        runCompile(STARTER_EXAMPLE);
      } catch (e) {
        if (destroyed) return;
        loadError = e instanceof Error ? e.message : String(e);
      }
    }

    setup();

    return () => {
      destroyed = true;
      editorView?.destroy();
      clearTimeout(parseTimer);
      clearTimeout(compileTimer);
    };
  });

  // --- Source change handler ---

  function onSourceChange(source: string) {
    // Fast parse (50ms debounce)
    clearTimeout(parseTimer);
    parseTimer = setTimeout(() => {
      if (!isReady()) return;
      const result = parseOnly(source);
      parseValid = result.success;
    }, PARSE_DEBOUNCE_MS);

    // Full compile (300ms debounce)
    clearTimeout(compileTimer);
    compileTimer = setTimeout(() => {
      runCompile(source);
    }, COMPILE_DEBOUNCE_MS);
  }

  function runCompile(source: string) {
    if (!isReady()) return;
    const seq = ++compileSeq;
    const t0 = performance.now();
    const result = compileSource(source);
    const elapsed = performance.now() - t0;
    // Discard if a newer compile has started
    if (seq !== compileSeq) return;
    compileResult = result;
    compileTimeMs = elapsed;
  }

  // --- Diagnostic click → editor scroll ---

  function handleDiagnosticClick(line: number, col: number) {
    if (!editorView) return;
    // Switch to editor on mobile
    mobileTab = 'editor';
    // Convert 1-indexed line to CodeMirror position
    const lineInfo = editorView.state.doc.line(Math.min(line, editorView.state.doc.lines));
    // Convert byte column to character offset
    const charCol = byteColToCharCol(lineInfo.text, col);
    const pos = lineInfo.from + Math.min(charCol - 1, lineInfo.length);
    editorView.dispatch({
      selection: { anchor: pos },
      scrollIntoView: true,
    });
    editorView.focus();
  }

  // --- Split pane drag ---

  function onDividerPointerDown(e: PointerEvent) {
    if (!playgroundEl) return;
    e.preventDefault();
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);

    const rect = playgroundEl.getBoundingClientRect();

    function onMove(ev: PointerEvent) {
      const pct = ((ev.clientX - rect.left) / rect.width) * 100;
      splitPercent = Math.max(25, Math.min(75, pct));
    }

    function onUp() {
      target.removeEventListener('pointermove', onMove);
      target.removeEventListener('pointerup', onUp);
    }

    target.addEventListener('pointermove', onMove);
    target.addEventListener('pointerup', onUp);
  }
</script>

<div
  class="playground"
  bind:this={playgroundEl}
  style="--split: {splitPercent}%"
  role="region"
  aria-label="Urd Playground"
>
  <!-- Mobile tab bar -->
  <div class="mobile-tabs">
    <button
      class="tab"
      class:active={mobileTab === 'editor'}
      onclick={() => mobileTab = 'editor'}
    >
      Editor
    </button>
    <button
      class="tab"
      class:active={mobileTab === 'output'}
      onclick={() => mobileTab = 'output'}
    >
      Output
      {#if compileResult && !compileResult.success}
        <span class="tab-badge error">
          {compileResult.diagnostics.length}
        </span>
      {/if}
    </button>
    <span class="parse-indicator" class:valid={parseValid} class:invalid={!parseValid}
      title={parseValid ? 'Syntax valid' : 'Syntax errors detected'}
    ></span>
  </div>

  <!-- Editor pane -->
  <div
    class="pane editor-pane"
    class:mobile-hidden={mobileTab !== 'editor'}
    role="region"
    aria-label="Schema Markdown editor"
  >
    <div class="editor-mount" bind:this={editorContainer}></div>
  </div>

  <!-- Divider (desktop only) -->
  <div
    class="divider"
    bind:this={dividerEl}
    onpointerdown={onDividerPointerDown}
    role="separator"
    aria-orientation="vertical"
  ></div>

  <!-- Output pane -->
  <div
    class="pane output-pane-container"
    class:mobile-hidden={mobileTab !== 'output'}
    role="region"
    aria-label="Compilation output"
  >
    <OutputPane
      {compileResult}
      {compileTimeMs}
      {compilerReady}
      {loadError}
      onDiagnosticClick={handleDiagnosticClick}
    />
  </div>
</div>

<style>
  .playground {
    display: grid;
    grid-template-columns: var(--split) 4px 1fr;
    grid-template-rows: 1fr;
    height: 70vh;
    min-height: 400px;
    max-height: 800px;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--deep);
  }

  /* --- Mobile tabs --- */

  .mobile-tabs {
    display: none;
    grid-column: 1 / -1;
    align-items: center;
    gap: 0;
    padding: 0 8px;
    background: var(--raise);
    border-bottom: 1px solid var(--border);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border: none;
    background: none;
    color: var(--faint);
    font-family: var(--display);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab.active {
    color: var(--text);
    border-bottom-color: var(--gold);
  }

  .tab-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    border-radius: 9px;
    font-size: 10px;
    font-family: var(--mono);
    font-weight: 600;
  }

  .tab-badge.error {
    background: var(--rose-dim);
    color: var(--rose);
  }

  .parse-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-left: auto;
    transition: background 0.2s;
  }

  .parse-indicator.valid { background: var(--green); }
  .parse-indicator.invalid { background: var(--rose); }

  /* --- Panes --- */

  .pane {
    overflow: hidden;
    min-width: 0;
  }

  .editor-pane {
    display: flex;
    flex-direction: column;
  }

  .editor-mount {
    flex: 1;
    overflow: hidden;
  }

  /* Make CodeMirror fill the container */
  .editor-mount :global(.cm-editor) {
    height: 100%;
  }

  .editor-mount :global(.cm-scroller) {
    overflow: auto;
  }

  .output-pane-container {
    display: flex;
    flex-direction: column;
  }

  /* --- Divider --- */

  .divider {
    background: var(--border);
    cursor: col-resize;
    transition: background 0.15s;
    touch-action: none;
  }

  .divider:hover {
    background: var(--gold-dim);
  }

  /* --- Responsive --- */

  @media (max-width: 1023px) {
    .playground {
      grid-template-columns: var(--split) 4px 1fr;
      --split: 55%;
    }
  }

  @media (max-width: 767px) {
    .playground {
      display: flex;
      flex-direction: column;
      grid-template-columns: unset;
    }

    .mobile-tabs {
      display: flex;
    }

    .divider {
      display: none;
    }

    .mobile-hidden {
      display: none;
    }

    .pane {
      flex: 1;
      min-height: 0;
    }
  }
</style>
