<script lang="ts">
  /**
   * DiagnosticInspector — shows details of the selected diagnostic.
   *
   * Subscribes to selection.primary. When a diagnostic is selected, shows its
   * severity badge, code, full message, location, and severity level.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext, type SelectionState } from '$lib/framework/selection/SelectionContext';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import InspectorPanel from './_shared/InspectorPanel.svelte';
  import InspectorSection from './_shared/InspectorSection.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let selectedId: string | null = $state(null);
  let selectedData: Record<string, unknown> | null = $state(null);

  let severity: string = $state('');
  let code: string = $state('');
  let message: string = $state('');
  let file: string = $state('');
  let line: number = $state(0);
  let col: number = $state(0);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    updateFromSelection(selectionContext.state);

    unsubscribers.push(
      selectionContext.subscribe(updateFromSelection)
    );

    unsubscribers.push(
      bus.subscribe('compiler.completed', () => {
        if (selectedId) {
          refreshData();
        }
      })
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function updateFromSelection(state: SelectionState): void {
    if (state.items.length > 0 && state.items[0].kind === 'diagnostic') {
      const item = state.items[0];
      selectedId = item.id;
      selectedData = (item.data as Record<string, unknown>) ?? null;
      refreshData();
    } else {
      selectedId = null;
      selectedData = null;
      severity = '';
      code = '';
      message = '';
      file = '';
      line = 0;
      col = 0;
    }
  }

  function refreshData(): void {
    if (!selectedId || !selectedData) {
      severity = '';
      code = '';
      message = '';
      file = '';
      line = 0;
      col = 0;
      return;
    }

    severity = (selectedData.severity as string) ?? '';
    code = (selectedData.code as string) ?? '';
    message = (selectedData.message as string) ?? '';
    file = (selectedData.file as string) ?? '';
    line = (selectedData.line as number) ?? 0;
    col = (selectedData.col as number) ?? 0;
  }

  function severityClass(sev: string): string {
    switch (sev) {
      case 'error': return 'forge-diagnostic-inspector__severity--error';
      case 'warning': return 'forge-diagnostic-inspector__severity--warning';
      case 'info': return 'forge-diagnostic-inspector__severity--info';
      default: return '';
    }
  }

  function goToSource(): void {
    if (file) {
      navigationBroker.navigate({
        targetViewId: 'urd.codeEditor',
        params: { path: file, line },
      });
    }
  }
</script>

<InspectorPanel
  title="Diagnostic Inspector"
  emptyMessage="Select a diagnostic to inspect"
  hasContent={selectedId !== null}
>
  {#if selectedId}
    <div class="forge-diagnostic-inspector">
      <div class="forge-diagnostic-inspector__header">
        <span class="forge-diagnostic-inspector__kind {severityClass(severity)}">{severity}</span>
        <span class="forge-diagnostic-inspector__name">{code}</span>
      </div>

      {#if file}
        <button
          class="forge-diagnostic-inspector__source-link"
          onclick={goToSource}
        >
          {file}:{line}:{col} →
        </button>
      {/if}

      <InspectorSection title="Message">
        <p class="forge-diagnostic-inspector__message">{message}</p>
      </InspectorSection>

      {#if file}
        <InspectorSection title="Location">
          <dl class="forge-diagnostic-inspector__props">
            <dt class="forge-diagnostic-inspector__key">file</dt>
            <dd class="forge-diagnostic-inspector__value">{file}</dd>
            <dt class="forge-diagnostic-inspector__key">line</dt>
            <dd class="forge-diagnostic-inspector__value">{line}</dd>
            <dt class="forge-diagnostic-inspector__key">column</dt>
            <dd class="forge-diagnostic-inspector__value">{col}</dd>
          </dl>
        </InspectorSection>
      {/if}

      <InspectorSection title="Severity">
        <dl class="forge-diagnostic-inspector__props">
          <dt class="forge-diagnostic-inspector__key">level</dt>
          <dd class="forge-diagnostic-inspector__value">{severity}</dd>
          <dt class="forge-diagnostic-inspector__key">code</dt>
          <dd class="forge-diagnostic-inspector__value">{code}</dd>
        </dl>
      </InspectorSection>
    </div>
  {/if}
</InspectorPanel>

<style>
  .forge-diagnostic-inspector {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-diagnostic-inspector__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-md);
  }

  .forge-diagnostic-inspector__kind {
    padding: 1px 6px;
    border-radius: var(--forge-radius-sm);
    background-color: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
  }

  .forge-diagnostic-inspector__severity--error {
    background-color: var(--forge-status-error, #e74c3c);
    color: #fff;
  }

  .forge-diagnostic-inspector__severity--warning {
    background-color: var(--forge-status-warning, #f39c12);
    color: #fff;
  }

  .forge-diagnostic-inspector__severity--info {
    background-color: var(--forge-status-info, #3498db);
    color: #fff;
  }

  .forge-diagnostic-inspector__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
  }

  .forge-diagnostic-inspector__source-link {
    display: inline-block;
    margin-bottom: var(--forge-space-md);
    padding: 0;
    background: none;
    border: none;
    color: var(--forge-accent-primary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
    text-decoration: none;
  }

  .forge-diagnostic-inspector__source-link:hover {
    text-decoration: underline;
  }

  .forge-diagnostic-inspector__message {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
    white-space: pre-wrap;
  }

  .forge-diagnostic-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
  }

  .forge-diagnostic-inspector__key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-diagnostic-inspector__value {
    color: var(--forge-text-primary);
    margin: 0;
    word-break: break-word;
  }
</style>
