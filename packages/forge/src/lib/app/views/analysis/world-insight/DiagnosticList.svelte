<script lang="ts">
  /**
   * DiagnosticList — diagnostics grouped by file with severity icons.
   */

  import type { FileDiagnostics } from '$lib/app/projections/diagnostics-by-file';
  import type { Diagnostic } from '$lib/app/compiler/types';

  interface Props {
    diagnosticsByFile: FileDiagnostics[] | null;
    filterText: string;
    onNavigate: (path: string, line: number) => void;
  }

  let { diagnosticsByFile, filterText, onNavigate }: Props = $props();

  let filtered = $derived.by(() => {
    if (!diagnosticsByFile) return [];
    if (!filterText) return diagnosticsByFile;

    const q = filterText.toLowerCase();
    const result: FileDiagnostics[] = [];

    for (const fileDiag of diagnosticsByFile) {
      const matching = fileDiag.diagnostics.filter(
        (d) =>
          d.message.toLowerCase().includes(q) ||
          d.code.toLowerCase().includes(q) ||
          fileDiag.file.toLowerCase().includes(q),
      );
      if (matching.length > 0) {
        result.push({
          ...fileDiag,
          diagnostics: matching,
          errorCount: matching.filter((d) => d.severity === 'error').length,
          warningCount: matching.filter((d) => d.severity === 'warning').length,
          infoCount: matching.filter((d) => d.severity === 'info').length,
        });
      }
    }
    return result;
  });

  function severityIcon(severity: string): string {
    switch (severity) {
      case 'error': return '\u26CC';
      case 'warning': return '\u25B2';
      default: return '\u2139';
    }
  }

  function handleDiagClick(diag: Diagnostic, file: string): void {
    const path = diag.span?.file ?? file;
    const line = diag.span?.startLine ?? 1;
    onNavigate(path, line);
  }
</script>

<div class="forge-diagnostic-list">
  {#if filtered.length === 0}
    <div class="forge-diagnostic-list__empty">No diagnostics</div>
  {:else}
    {#each filtered as fileDiag}
      <div class="forge-diagnostic-list__file-group">
        <div class="forge-diagnostic-list__file-header">
          <span class="forge-diagnostic-list__file-name">{fileDiag.file}</span>
          <span class="forge-diagnostic-list__file-count">{fileDiag.diagnostics.length}</span>
        </div>
        {#each fileDiag.diagnostics as diag}
          <button
            class="forge-diagnostic-list__row"
            onclick={() => handleDiagClick(diag, fileDiag.file)}
          >
            <span
              class="forge-diagnostic-list__icon"
              class:forge-diagnostic-list__icon--error={diag.severity === 'error'}
              class:forge-diagnostic-list__icon--warning={diag.severity === 'warning'}
              class:forge-diagnostic-list__icon--info={diag.severity === 'info'}
            >
              {severityIcon(diag.severity)}
            </span>
            <span class="forge-diagnostic-list__message">{diag.message}</span>
            <span class="forge-diagnostic-list__code">{diag.code}</span>
            {#if diag.span}
              <span class="forge-diagnostic-list__location">
                {diag.span.file}:{diag.span.startLine}
              </span>
            {/if}
          </button>
        {/each}
      </div>
    {/each}
  {/if}
</div>

<style>
  .forge-diagnostic-list {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-diagnostic-list__empty {
    padding: var(--forge-space-md);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    text-align: center;
  }

  .forge-diagnostic-list__file-group {
    margin-bottom: var(--forge-space-xs);
  }

  .forge-diagnostic-list__file-header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-xs) var(--forge-space-sm);
    color: var(--forge-text-muted);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .forge-diagnostic-list__file-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-diagnostic-list__file-count {
    font-size: 10px;
    padding: 0 var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 15%, transparent);
    color: var(--forge-text-secondary);
  }

  .forge-diagnostic-list__row {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    width: 100%;
    height: 24px;
    padding: 0 var(--forge-space-sm) 0 var(--forge-space-md);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
  }

  .forge-diagnostic-list__row:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-diagnostic-list__icon {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    font-size: 10px;
  }

  .forge-diagnostic-list__icon--error {
    color: var(--forge-status-error);
  }

  .forge-diagnostic-list__icon--warning {
    color: var(--forge-status-warning);
  }

  .forge-diagnostic-list__icon--info {
    color: var(--forge-text-muted);
  }

  .forge-diagnostic-list__message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--forge-text-secondary);
  }

  .forge-diagnostic-list__code {
    flex-shrink: 0;
    color: var(--forge-text-muted);
    font-size: 10px;
  }

  .forge-diagnostic-list__location {
    flex-shrink: 0;
    color: var(--forge-text-muted);
    font-size: 10px;
    margin-left: var(--forge-space-xs);
  }
</style>
