<script lang="ts">
  /**
   * CompilerOutputPanel — displays the raw JSON output from the compiler.
   *
   * Tabbed view with one tab per chunk (AST, Symbol Table, Fact Set,
   * Property Dependencies, Definitions, urdJson, Diagnostics) plus a
   * Header tab. Refreshes on compiler.completed.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { ResolvedCompilerOutput } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: CompilerOutputState | null;
    onStateChange: (newState: unknown) => void;
  }

  interface CompilerOutputState {
    activeTab: string;
    expandDepth: number;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  type TabId = 'header' | 'ast' | 'symbolTable' | 'factSet' | 'propertyDependencyIndex' | 'definitionIndex' | 'urdJson' | 'diagnostics';

  const tabs: { id: TabId; label: string }[] = [
    { id: 'header', label: 'Header' },
    { id: 'urdJson', label: 'urdJson' },
    { id: 'ast', label: 'AST' },
    { id: 'symbolTable', label: 'Symbols' },
    { id: 'factSet', label: 'Facts' },
    { id: 'propertyDependencyIndex', label: 'Prop Deps' },
    { id: 'definitionIndex', label: 'Definitions' },
    { id: 'diagnostics', label: 'Diagnostics' },
  ];

  let activeTab: TabId = $state(zoneState?.activeTab as TabId ?? 'urdJson');
  let expandDepth: number = $state(zoneState?.expandDepth ?? 3);
  let output: ResolvedCompilerOutput | null = $state(null);
  let jsonText: string = $state('');
  let copyFeedback: boolean = $state(false);

  const unsubscribers: (() => void)[] = [];

  function refreshData(): void {
    output = projectionRegistry.get<ResolvedCompilerOutput>('urd.projection.compilerOutput');
    updateJsonText();
  }

  function updateJsonText(): void {
    if (!output) {
      jsonText = '// No compiler output available.\n// Open a project with .urd.md files.';
      return;
    }

    let data: unknown;
    switch (activeTab) {
      case 'header':
        data = output.header;
        break;
      case 'ast':
        data = output.ast;
        break;
      case 'symbolTable':
        data = output.symbolTable;
        break;
      case 'factSet':
        data = output.factSet;
        break;
      case 'propertyDependencyIndex':
        data = output.propertyDependencyIndex;
        break;
      case 'definitionIndex':
        data = output.definitionIndex;
        break;
      case 'urdJson':
        // Show raw (un-normalised) if available, otherwise normalised
        data = output.rawUrdJson ?? output.urdJson;
        break;
      case 'diagnostics':
        data = output.diagnostics;
        break;
    }

    try {
      jsonText = JSON.stringify(data, null, 2);
    } catch {
      jsonText = '// Could not serialise output.';
    }
  }

  function switchTab(tab: TabId): void {
    activeTab = tab;
    updateJsonText();
    persistState();
  }

  function setExpandDepth(depth: number): void {
    expandDepth = depth;
    persistState();
  }

  function persistState(): void {
    onStateChange({ activeTab, expandDepth });
  }

  async function copyToClipboard(): Promise<void> {
    try {
      await navigator.clipboard.writeText(jsonText);
      copyFeedback = true;
      setTimeout(() => { copyFeedback = false; }, 1500);
    } catch {
      // clipboard API may not be available
    }
  }

  function diagnosticCount(): { errors: number; warnings: number; info: number } {
    if (!output?.diagnostics) return { errors: 0, warnings: 0, info: 0 };
    const d = output.diagnostics;
    return {
      errors: d.filter((x) => x.severity === 'error').length,
      warnings: d.filter((x) => x.severity === 'warning').length,
      info: d.filter((x) => x.severity === 'info').length,
    };
  }

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', () => refreshData()));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
    persistState();
  });
</script>

<div class="forge-json-panel">
  <!-- Toolbar -->
  <div class="forge-json-panel__toolbar">
    <div class="forge-json-panel__tabs">
      {#each tabs as tab}
        <button
          class="forge-json-panel__tab"
          class:forge-json-panel__tab--active={activeTab === tab.id}
          onclick={() => switchTab(tab.id)}
        >
          {tab.label}
          {#if tab.id === 'diagnostics' && output?.diagnostics}
            {@const counts = diagnosticCount()}
            {#if counts.errors > 0}
              <span class="forge-json-panel__badge forge-json-panel__badge--error">{counts.errors}</span>
            {/if}
            {#if counts.warnings > 0}
              <span class="forge-json-panel__badge forge-json-panel__badge--warning">{counts.warnings}</span>
            {/if}
          {/if}
        </button>
      {/each}
    </div>
    <div class="forge-json-panel__actions">
      <button
        class="forge-json-panel__action-btn"
        onclick={copyToClipboard}
        title="Copy to clipboard"
      >
        {copyFeedback ? 'Copied' : 'Copy'}
      </button>
    </div>
  </div>

  <!-- JSON content -->
  <div class="forge-json-panel__content">
    <pre class="forge-json-panel__pre"><code class="forge-json-panel__code">{jsonText}</code></pre>
  </div>

  <!-- Status bar -->
  {#if output}
    <div class="forge-json-panel__status">
      <span class="forge-json-panel__status-item">
        {output.header.durationMs}ms
      </span>
      <span class="forge-json-panel__status-item">
        {output.header.worldCounts.entities} entities
      </span>
      <span class="forge-json-panel__status-item">
        {output.header.worldCounts.locations} locations
      </span>
      <span class="forge-json-panel__status-item">
        {jsonText.split('\n').length} lines
      </span>
    </div>
  {/if}
</div>

<style>
  .forge-json-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-json-panel__toolbar {
    display: flex;
    align-items: center;
    height: 32px;
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-json-panel__tabs {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    gap: 0;
  }

  .forge-json-panel__tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 10px;
    height: 32px;
    border: none;
    border-bottom: 2px solid transparent;
    background: none;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
    white-space: nowrap;
    transition: color 0.15s, border-color 0.15s;
  }

  .forge-json-panel__tab:hover {
    color: var(--forge-text-primary);
  }

  .forge-json-panel__tab--active {
    color: var(--forge-text-primary);
    border-bottom-color: var(--forge-accent, #7c8cff);
  }

  .forge-json-panel__badge {
    display: inline-block;
    min-width: 16px;
    padding: 0 4px;
    border-radius: 8px;
    font-size: 10px;
    font-weight: 600;
    text-align: center;
    line-height: 16px;
  }

  .forge-json-panel__badge--error {
    background: var(--forge-severity-error, #e94560);
    color: #fff;
  }

  .forge-json-panel__badge--warning {
    background: var(--forge-severity-warning, #e6a817);
    color: #fff;
  }

  .forge-json-panel__actions {
    display: flex;
    align-items: center;
    padding: 0 var(--forge-space-sm);
    flex-shrink: 0;
  }

  .forge-json-panel__action-btn {
    padding: 2px 8px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-json-panel__action-btn:hover {
    background: var(--forge-bg-hover);
    color: var(--forge-text-primary);
  }

  .forge-json-panel__content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    background: var(--forge-bg-primary);
  }

  .forge-json-panel__pre {
    margin: 0;
    padding: var(--forge-space-md);
    white-space: pre;
    word-break: normal;
    overflow-wrap: normal;
  }

  .forge-json-panel__code {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    line-height: 1.6;
    color: var(--forge-text-primary);
    tab-size: 2;
  }

  .forge-json-panel__status {
    display: flex;
    align-items: center;
    gap: var(--forge-space-md);
    height: 22px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-top: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-json-panel__status-item {
    font-size: 10px;
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }
</style>
