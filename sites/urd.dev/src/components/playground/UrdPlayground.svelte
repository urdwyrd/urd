<script lang="ts">
  import { onMount } from 'svelte';
  import OutputPane from './OutputPane.svelte';
  import FactSetView from './FactSetView.svelte';
  import PropertyDependencyView from './PropertyDependencyView.svelte';
  import LocationGraph from './LocationGraph.svelte';
  import DialogueGraph from './DialogueGraph.svelte';
  import {
    initCompiler,
    compileSource,
    parseOnly,
    isReady,
    compilerVersion,
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
    trust: int(0,100) = 0
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
  let factsOpen = $state(true);
  let factsExpanded = $state(false);
  let analysisTab: 'properties' | 'location' | 'dialogue' | 'facts' = $state('properties');

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

  function getInitialSource(): string {
    try {
      const hash = window.location.hash;
      if (hash.startsWith('#code=')) {
        const raw = decodeURIComponent(hash.slice(6));
        // Accept both standard and URL-safe base64
        const standard = raw.replace(/-/g, '+').replace(/_/g, '/');
        return atob(standard);
      }
    } catch { /* ignore decode failures */ }
    return STARTER_EXAMPLE;
  }

  onMount(() => {
    let destroyed = false;
    let themeObserver: MutationObserver | undefined;

    async function setup() {
      const initialSource = getInitialSource();

      // 1. Mount CodeMirror
      const { EditorView, keymap, placeholder, lineNumbers, highlightActiveLine, drawSelection } = await import('@codemirror/view');
      const { EditorState, Compartment } = await import('@codemirror/state');
      const { defaultKeymap, history, historyKeymap, indentWithTab } = await import('@codemirror/commands');
      const { bracketMatching, indentOnInput } = await import('@codemirror/language');
      const { lintGutter } = await import('@codemirror/lint');
      const { urdLanguage, gloamingTheme, parchmentTheme, urdHighlight } = await import('./codemirror-urd');

      if (destroyed) return;

      // Theme compartment for dynamic switching
      const themeCompartment = new Compartment();
      const isParchment = document.documentElement.getAttribute('data-theme') === 'parchment';

      editorView = new EditorView({
        parent: editorContainer!,
        state: EditorState.create({
          doc: initialSource,
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
            themeCompartment.of(isParchment ? parchmentTheme : gloamingTheme),
            urdHighlight,
            EditorView.updateListener.of((update) => {
              if (update.docChanged) {
                onSourceChange(update.state.doc.toString());
              }
            }),
          ],
        }),
      });

      // Watch for theme changes on <html data-theme>
      themeObserver = new MutationObserver(() => {
        if (!editorView) return;
        const parchment = document.documentElement.getAttribute('data-theme') === 'parchment';
        editorView.dispatch({
          effects: themeCompartment.reconfigure(parchment ? parchmentTheme : gloamingTheme),
        });
      });
      themeObserver.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ['data-theme'],
      });

      // 2. Load WASM compiler
      try {
        await initCompiler();
        if (destroyed) return;
        compilerReady = true;

        // Inject version into the static footer
        const metaEl = document.getElementById('compiler-meta');
        if (metaEl) {
          const ver = compilerVersion();
          metaEl.innerHTML = metaEl.innerHTML.replace(
            'urd-compiler',
            `urd-compiler v${ver}`,
          );
          // Append changelog link
          const link = document.createElement('a');
          link.href = 'https://github.com/urdwyrd/urd/blob/main/packages/compiler/CHANGELOG.md';
          link.target = '_blank';
          link.rel = 'noopener';
          link.className = 'changelog-link';
          link.textContent = 'changelog';
          metaEl.appendChild(link);
        }

        // Compile the initial source
        runCompile(initialSource);
      } catch (e) {
        if (destroyed) return;
        loadError = e instanceof Error ? e.message : String(e);
      }
    }

    setup();

    return () => {
      destroyed = true;
      themeObserver?.disconnect();
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
  role="region"
  aria-label="Urd Playground"
>
  <!-- Main area: editor + output side by side -->
  <div class="main-area" style="--split: {splitPercent}%">
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

  <!-- Facts panel -->
  {#if compileResult?.facts}
    <div class="facts-panel" class:facts-open={factsOpen}>
      <button class="facts-header" onclick={() => factsOpen = !factsOpen}>
        <span class="facts-toggle">{factsOpen ? '▾' : '▸'}</span>
        <span class="facts-title">Analysis</span>
        <span class="facts-count">{
          compileResult.facts.exits.length +
          compileResult.facts.choices.length +
          compileResult.facts.reads.length +
          compileResult.facts.writes.length +
          compileResult.facts.jumps.length +
          compileResult.facts.rules.length
        } facts</span>
        {#if factsOpen}
          <button
            class="facts-size-btn"
            title={factsExpanded ? 'Reduce' : 'Expand'}
            onclick={(e) => { e.stopPropagation(); factsExpanded = !factsExpanded; }}
          >{factsExpanded ? '▵' : '▿'}</button>
        {/if}
      </button>
      {#if factsOpen}
        <div class="analysis-tabs" role="tablist">
          <button
            class="analysis-tab"
            class:analysis-tab-active={analysisTab === 'properties'}
            role="tab"
            aria-selected={analysisTab === 'properties'}
            onclick={() => analysisTab = 'properties'}
          >Properties</button>
          <button
            class="analysis-tab"
            class:analysis-tab-active={analysisTab === 'location'}
            role="tab"
            aria-selected={analysisTab === 'location'}
            onclick={() => analysisTab = 'location'}
          >Location</button>
          <button
            class="analysis-tab"
            class:analysis-tab-active={analysisTab === 'dialogue'}
            role="tab"
            aria-selected={analysisTab === 'dialogue'}
            onclick={() => analysisTab = 'dialogue'}
          >Dialogue</button>
          <button
            class="analysis-tab"
            class:analysis-tab-active={analysisTab === 'facts'}
            role="tab"
            aria-selected={analysisTab === 'facts'}
            onclick={() => analysisTab = 'facts'}
          >Facts</button>
        </div>
        <div class="facts-body" class:facts-expanded={factsExpanded}>
          {#if analysisTab === 'properties' && compileResult.property_index}
            <PropertyDependencyView
              propertyIndex={compileResult.property_index}
              facts={compileResult.facts}
              onDiagnosticClick={handleDiagnosticClick}
            />
          {:else if analysisTab === 'location' && compileResult.facts}
            <LocationGraph
              facts={compileResult.facts}
              worldJson={compileResult.world}
              diagnostics={compileResult.diagnostics}
            />
          {:else if analysisTab === 'dialogue' && compileResult.facts}
            <DialogueGraph
              facts={compileResult.facts}
              diagnostics={compileResult.diagnostics}
              onDiagnosticClick={handleDiagnosticClick}
            />
          {:else}
            <FactSetView facts={compileResult.facts} onDiagnosticClick={handleDiagnosticClick} />
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .playground {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--deep);
  }

  .main-area {
    display: grid;
    grid-template-columns: var(--split) 4px 1fr;
    grid-template-rows: 1fr;
    height: 70vh;
    min-height: 400px;
    max-height: 800px;
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

  /* --- Facts panel --- */

  .facts-panel {
    border-top: 1px solid var(--border);
  }

  .facts-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 12px;
    border: none;
    background: var(--raise);
    color: var(--text);
    font-family: var(--display);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .facts-header:hover {
    background: var(--surface);
  }

  .facts-toggle {
    color: var(--faint);
    width: 10px;
    font-size: 10px;
  }

  .facts-title {
    text-transform: uppercase;
  }

  .facts-count {
    color: var(--faint);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 400;
    margin-left: auto;
  }

  .facts-size-btn {
    padding: 1px 6px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface);
    color: var(--faint);
    font-family: var(--mono);
    font-size: 10px;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
    margin-left: 6px;
  }

  .facts-size-btn:hover {
    border-color: var(--gold-dim);
    color: var(--text);
  }

  .analysis-tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border);
    background: var(--raise);
  }

  .analysis-tab {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--faint);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    padding: 5px 12px;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }

  .analysis-tab:hover {
    color: var(--dim);
  }

  .analysis-tab-active {
    color: var(--gold);
    border-bottom-color: var(--gold);
  }

  .analysis-tab:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: -2px;
  }

  .facts-body {
    max-height: 280px;
    overflow: auto;
    border-top: 1px solid var(--border);
    transition: max-height 0.2s ease;
  }

  .facts-body.facts-expanded {
    max-height: 70vh;
  }

  /* --- Responsive --- */

  @media (max-width: 1023px) {
    .main-area {
      grid-template-columns: var(--split) 4px 1fr;
      --split: 55%;
    }
  }

  @media (max-width: 767px) {
    .main-area {
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

    .facts-body {
      max-height: 200px;
    }
  }
</style>
