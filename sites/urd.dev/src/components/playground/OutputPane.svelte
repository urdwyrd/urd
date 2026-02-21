<script lang="ts">
  import { onMount } from 'svelte';
  import type { CompileResult, Diagnostic } from './compiler-bridge';
  import { validateSchema } from './schema-validator';

  // --- Props ---

  interface Props {
    compileResult: CompileResult | null;
    compileTimeMs: number;
    compilerReady: boolean;
    loadError: string | null;
    onDiagnosticClick?: (line: number, col: number) => void;
  }

  let {
    compileResult,
    compileTimeMs,
    compilerReady,
    loadError,
    onDiagnosticClick,
  }: Props = $props();

  // --- State ---

  let outputContainer: HTMLDivElement | undefined = $state();
  let copied = $state(false);
  let validationState: 'idle' | 'loading' | 'valid' | 'invalid' = $state('idle');
  let validationErrors: string[] = $state([]);
  let autoValidate = $state(false);

  onMount(() => {
    const stored = localStorage.getItem('urd-auto-validate');
    if (stored !== null) autoValidate = stored === 'true';
  });

  // Auto-validate when compile result changes (if enabled)
  $effect(() => {
    if (!compileResult?.success || !compileResult.world) {
      validationState = 'idle';
      validationErrors = [];
      return;
    }
    validationState = 'idle';
    validationErrors = [];
    if (autoValidate) {
      runValidation();
    }
  });

  // --- Derived ---

  let prettyJson = $derived.by(() => {
    if (!compileResult?.success || !compileResult.world) return '';
    try {
      return JSON.stringify(JSON.parse(compileResult.world), null, 2);
    } catch {
      return compileResult.world;
    }
  });

  let highlightedJson = $derived.by(() => {
    if (!prettyJson) return '';
    return highlightJson(prettyJson);
  });

  let diagnostics = $derived(compileResult?.diagnostics ?? []);
  let hasErrors = $derived(diagnostics.some(d => d.severity === 'error'));
  let warnings = $derived(diagnostics.filter(d => d.severity !== 'error'));

  let formattedTime = $derived(
    compileTimeMs < 1
      ? `<1ms (${Math.round(compileTimeMs * 1000)}µs)`
      : `${Math.round(compileTimeMs)}ms`
  );

  // --- Actions ---

  async function copyJson() {
    if (!prettyJson) return;
    try {
      await navigator.clipboard.writeText(prettyJson);
      copied = true;
      setTimeout(() => copied = false, 2000);
    } catch {
      // Clipboard API not available
    }
  }

  async function runValidation() {
    if (!compileResult?.world) return;
    validationState = 'loading';
    try {
      const result = await validateSchema(compileResult.world);
      validationState = result.valid ? 'valid' : 'invalid';
      validationErrors = result.errors;
    } catch {
      validationState = 'invalid';
      validationErrors = ['Schema validation failed to load.'];
    }
  }

  function setAutoValidate(on: boolean) {
    autoValidate = on;
    localStorage.setItem('urd-auto-validate', String(on));
    if (on && compileResult?.success && compileResult.world) {
      runValidation();
    }
    if (!on) {
      validationState = 'idle';
      validationErrors = [];
    }
  }

  function handleDiagnosticClick(d: Diagnostic) {
    onDiagnosticClick?.(d.span.start_line, d.span.start_col);
  }

  function severityIcon(severity: string): string {
    switch (severity) {
      case 'error': return '◆';
      case 'warning': return '▸';
      default: return '→';
    }
  }

  function esc(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  function highlightJson(json: string): string {
    // Tokenise JSON for syntax highlighting.
    // Matches: strings, numbers, booleans, null, and structural chars.
    return json.replace(
      /("(?:[^"\\]|\\.)*")\s*:|("(?:[^"\\]|\\.)*")|(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)\b|(true|false)|(null)|([{}\[\]])|([,:])/g,
      (match, key, str, num, bool, nil, bracket, punct) => {
        if (key) return `<span class="j-key">${esc(key)}</span>:`;
        if (str) return `<span class="j-str">${esc(str)}</span>`;
        if (num) return `<span class="j-num">${esc(num)}</span>`;
        if (bool) return `<span class="j-bool">${esc(bool)}</span>`;
        if (nil) return `<span class="j-null">${esc(nil)}</span>`;
        if (bracket) return `<span class="j-bracket">${bracket}</span>`;
        if (punct) return `<span class="j-punct">${punct}</span>`;
        return esc(match);
      }
    );
  }
</script>

<div class="output-pane" bind:this={outputContainer}>
  {#if loadError}
    <!-- WASM load failure -->
    <div class="output-state error-state">
      <span class="state-icon">◆</span>
      <p>Compiler failed to load. Try refreshing the page.</p>
      <p class="error-detail">{loadError}</p>
    </div>
  {:else if !compilerReady}
    <!-- Loading state -->
    <div class="output-state loading-state">
      <span class="loading-dot"></span>
      <p>Initialising compiler…</p>
    </div>
  {:else if !compileResult}
    <!-- No result yet -->
    <div class="output-state loading-state">
      <p>Compiling…</p>
    </div>
  {:else if compileResult.success}
    <!-- Compiled JSON -->
    <div class="output-header">
      <span class="compile-time">
        Compiled in {formattedTime}
        {#if warnings.length > 0}
          <span class="warning-badge">{warnings.length} {warnings.length === 1 ? 'warning' : 'warnings'}</span>
        {/if}
      </span>
      <div class="header-actions">
        <span class="validate-label">Schema Check</span>
        <div class="validate-toggle" role="group" aria-label="Schema validation">
          <button
            class="validate-toggle-btn"
            class:active={!autoValidate}
            onclick={() => setAutoValidate(false)}
          >Off</button>
          <button
            class="validate-toggle-btn"
            class:active={autoValidate}
            onclick={() => setAutoValidate(true)}
          >On</button>
        </div>
        <span
          class="validate-light"
          class:validate-light-valid={validationState === 'valid'}
          class:validate-light-invalid={validationState === 'invalid'}
          title={validationState === 'valid' ? 'Schema valid' : validationState === 'invalid' ? 'Schema validation failed' : 'Inactive'}
        ></span>
        <button class="copy-btn" onclick={copyJson}>
          {copied ? 'Copied' : 'Copy JSON'}
        </button>
      </div>
    </div>
    {#if warnings.length > 0}
      <div class="warnings-list" role="list" aria-label="Compilation warnings">
        {#each warnings as d}
          <button
            class="diagnostic-row severity-{d.severity}"
            role="listitem"
            onclick={() => handleDiagnosticClick(d)}
          >
            <span class="diag-icon">{severityIcon(d.severity)}</span>
            <span class="diag-code">{d.code}</span>
            <span class="diag-line">L{d.span.start_line}</span>
            <span class="diag-message">{d.message}</span>
          </button>
        {/each}
      </div>
    {/if}
    <div class="json-output">
      <pre><code>{@html highlightedJson}</code></pre>
    </div>
  {:else}
    <!-- Diagnostics -->
    <div class="output-header">
      <span class="diagnostic-count">
        {diagnostics.length} {diagnostics.length === 1 ? 'diagnostic' : 'diagnostics'}
      </span>
      {#if compileTimeMs > 0}
        <span class="compile-time">{formattedTime}</span>
      {/if}
    </div>
    <div class="diagnostics-list" role="list" aria-label="Compilation diagnostics">
      {#each diagnostics as d}
        <button
          class="diagnostic-row severity-{d.severity}"
          role="listitem"
          onclick={() => handleDiagnosticClick(d)}
        >
          <span class="diag-icon">{severityIcon(d.severity)}</span>
          <span class="diag-code">{d.code}</span>
          <span class="diag-line">L{d.span.start_line}</span>
          <span class="diag-message">{d.message}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .output-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--deep);
    font-family: var(--mono);
    font-size: 13px;
    color: var(--text);
  }

  /* --- States --- */

  .output-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: var(--faint);
    font-family: var(--body);
    font-size: 14px;
  }

  .state-icon {
    font-size: 20px;
    color: var(--rose);
  }

  .error-state p {
    margin: 0;
    text-align: center;
    max-width: 300px;
  }

  .error-detail {
    font-size: 12px;
    color: var(--faint);
    font-family: var(--mono);
  }

  .loading-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--gold);
    animation: pulse 1.5s ease-in-out infinite;
  }

  @media (prefers-reduced-motion: reduce) {
    .loading-dot {
      animation: none;
      opacity: 0.6;
    }
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 1; }
  }

  /* --- Header --- */

  .output-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--raise);
    flex-shrink: 0;
  }

  .compile-time {
    font-size: 11px;
    color: var(--faint);
    font-family: var(--mono);
  }

  .diagnostic-count {
    font-size: 12px;
    color: var(--dim);
    font-family: var(--display);
    font-weight: 500;
  }

  .copy-btn {
    padding: 2px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    color: var(--dim);
    font-family: var(--mono);
    font-size: 11px;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .copy-btn:hover {
    border-color: var(--gold-dim);
    color: var(--text);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .validate-label {
    font-family: var(--display);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.02em;
    color: var(--faint);
    flex-shrink: 0;
  }

  .validate-toggle {
    display: flex;
    gap: 3px;
    flex-shrink: 0;
    background: var(--raise);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px;
  }

  .validate-toggle-btn {
    display: flex;
    align-items: center;
    font-family: var(--display);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.02em;
    line-height: 1;
    color: var(--faint);
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 3px 8px;
    cursor: pointer;
    transition: color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease;
  }

  .validate-toggle-btn.active {
    color: var(--gold);
    background: var(--surface);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .validate-toggle-btn:hover:not(.active) {
    color: var(--dim);
    background: var(--surface);
  }

  .validate-light {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--gold-dim);
    flex-shrink: 0;
    transition: background 0.2s ease;
  }

  .validate-light-valid {
    background: var(--green-light);
  }

  .validate-light-invalid {
    background: var(--rose);
  }

  /* --- Warnings (success with diagnostics) --- */

  .warning-badge {
    margin-left: 8px;
    padding: 1px 6px;
    border-radius: 3px;
    background: color-mix(in srgb, var(--amber-light) 15%, transparent);
    color: var(--amber-light);
    font-size: 10px;
    font-weight: 500;
  }

  .warnings-list {
    border-bottom: 1px solid var(--border);
    padding: 2px 0;
    flex-shrink: 0;
    max-height: 30%;
    overflow: auto;
  }

  /* --- JSON output --- */

  .json-output {
    flex: 1;
    overflow: auto;
    padding: 12px;
  }

  .json-output pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--mono);
    font-size: 12px;
    line-height: 1.6;
    color: var(--dim);
  }

  /* JSON syntax highlighting */
  .json-output :global(.j-key) { color: var(--gold); }
  .json-output :global(.j-str) { color: var(--green-light); }
  .json-output :global(.j-num) { color: var(--amber-light); }
  .json-output :global(.j-bool) { color: var(--purple); }
  .json-output :global(.j-null) { color: var(--faint); font-style: italic; }
  .json-output :global(.j-bracket) { color: var(--dim); }
  .json-output :global(.j-punct) { color: var(--faint); }

  /* --- Diagnostics --- */

  .diagnostics-list {
    flex: 1;
    overflow: auto;
    padding: 4px 0;
  }

  .diagnostic-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    border: none;
    background: none;
    color: var(--text);
    font-family: var(--mono);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .diagnostic-row:hover {
    background: var(--surface);
  }

  .diag-icon {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
  }

  .severity-error .diag-icon { color: var(--rose); }
  .severity-warning .diag-icon { color: var(--amber-light); }
  .severity-info .diag-icon { color: var(--blue); }

  .diag-code {
    flex-shrink: 0;
    color: var(--faint);
    font-size: 11px;
    min-width: 52px;
  }

  .diag-line {
    flex-shrink: 0;
    color: var(--faint);
    font-size: 11px;
    min-width: 32px;
  }

  .diag-message {
    flex: 1;
    color: var(--dim);
    line-height: 1.4;
  }

  .severity-error .diag-message { color: var(--text); }
</style>
