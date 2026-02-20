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
  let autoValidate = $state(true);

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

  function toggleAutoValidate() {
    autoValidate = !autoValidate;
    localStorage.setItem('urd-auto-validate', String(autoValidate));
    if (autoValidate && compileResult?.success && compileResult.world) {
      runValidation();
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
      <span class="compile-time">Compiled in {formattedTime}</span>
      <div class="header-actions">
        <div class="validate-group">
          {#if validationState === 'loading'}
            <span class="validate-status">Validating…</span>
          {:else if validationState === 'valid'}
            <span class="validate-status validate-valid">Valid</span>
          {:else if validationState === 'invalid'}
            <button class="validate-status validate-invalid" onclick={runValidation}>Invalid</button>
          {:else if !autoValidate}
            <button class="validate-btn" onclick={runValidation}>Validate</button>
          {/if}
          <button
            class="auto-toggle"
            class:auto-toggle-on={autoValidate}
            onclick={toggleAutoValidate}
            title={autoValidate ? 'Auto-validate on' : 'Auto-validate off'}
            role="switch"
            aria-checked={autoValidate}
          >
            <span class="auto-toggle-knob"></span>
          </button>
        </div>
        <button class="copy-btn" onclick={copyJson}>
          {copied ? 'Copied' : 'Copy JSON'}
        </button>
      </div>
    </div>
    {#if validationState === 'invalid' && validationErrors.length > 0}
      <div class="validation-errors" role="list" aria-label="Schema validation errors">
        {#each validationErrors as err}
          <div class="validation-error-row" role="listitem">{err}</div>
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

  .validate-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .auto-toggle {
    position: relative;
    width: 26px;
    height: 14px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--surface);
    cursor: pointer;
    padding: 0;
    transition: background 0.2s, border-color 0.2s;
    flex-shrink: 0;
  }

  .auto-toggle-knob {
    position: absolute;
    top: 1px;
    left: 1px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--faint);
    transition: transform 0.2s, background 0.2s;
  }

  .auto-toggle-on {
    border-color: var(--green-light);
  }

  .auto-toggle-on .auto-toggle-knob {
    transform: translateX(12px);
    background: var(--green-light);
  }

  .auto-toggle:hover {
    border-color: var(--dim);
  }

  .auto-toggle-on:hover {
    border-color: var(--green-light);
  }

  @media (prefers-reduced-motion: reduce) {
    .auto-toggle-knob {
      transition: none;
    }
  }

  .validate-status {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    border: none;
    background: none;
    padding: 0;
  }

  .validate-btn {
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

  .validate-btn:hover {
    border-color: var(--gold-dim);
    color: var(--text);
  }

  .validate-valid {
    color: var(--green-light);
  }

  .validate-invalid {
    color: var(--rose);
    cursor: pointer;
  }

  .validate-invalid:hover {
    text-decoration: underline;
  }

  .validation-errors {
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--raise);
    max-height: 120px;
    overflow: auto;
  }

  .validation-error-row {
    font-family: var(--mono);
    font-size: 11px;
    line-height: 1.6;
    color: var(--rose);
    padding: 1px 0;
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
