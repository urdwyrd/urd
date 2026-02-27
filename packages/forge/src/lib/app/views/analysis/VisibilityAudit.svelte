<script lang="ts">
  /**
   * VisibilityAudit — lists properties with visibility constraints and access counts.
   *
   * Reads from the urd.projection.visibilityAudit projection, auto-refreshed
   * on compiler.completed. Displays each restricted property with its
   * visibility level, read count, and write count.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type { VisibilityEntry } from '$lib/app/projections/visibility-audit';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let entries: VisibilityEntry[] = $state([]);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshData(): void {
    entries = projectionRegistry.get<VisibilityEntry[]>('urd.projection.visibilityAudit') ?? [];
  }

  // Group by visibility level
  let grouped = $derived.by(() => {
    const groups = new Map<string, VisibilityEntry[]>();
    for (const entry of entries) {
      const list = groups.get(entry.visibility) ?? [];
      list.push(entry);
      groups.set(entry.visibility, list);
    }
    return [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  });

  function visibilityBadgeColour(vis: string): string {
    switch (vis) {
      case 'private': return 'var(--forge-status-error)';
      case 'protected': return 'var(--forge-status-warning)';
      case 'internal': return 'var(--forge-status-info, var(--forge-text-secondary))';
      default: return 'var(--forge-text-muted)';
    }
  }
</script>

<div class="forge-analysis-visibility">
  {#if entries.length === 0}
    <div class="forge-analysis-visibility__empty">
      No properties with visibility restrictions found
    </div>
  {:else}
    <div class="forge-analysis-visibility__list">
      {#each grouped as [visibility, items]}
        <div class="forge-analysis-visibility__group">
          <div class="forge-analysis-visibility__group-header">
            <span
              class="forge-analysis-visibility__badge"
              style:color={visibilityBadgeColour(visibility)}
            >
              {visibility}
            </span>
            <span class="forge-analysis-visibility__group-count">
              ({items.length})
            </span>
          </div>
          {#each items as entry}
            <div class="forge-analysis-visibility__entry">
              <span class="forge-analysis-visibility__name">
                {entry.entityType}.{entry.property}
              </span>
              <div class="forge-analysis-visibility__counts">
                <span class="forge-analysis-visibility__count-read"
                  title="Reads"
                >
                  R: {entry.readCount}
                </span>
                <span class="forge-analysis-visibility__count-write"
                  title="Writes"
                >
                  W: {entry.writeCount}
                </span>
              </div>
            </div>
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-analysis-visibility {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-visibility__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-visibility__list {
    padding: var(--forge-space-md);
  }

  .forge-analysis-visibility__group {
    margin-bottom: var(--forge-space-md);
  }

  .forge-analysis-visibility__group-header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-xs) 0;
    border-bottom: 1px solid var(--forge-border-zone);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-analysis-visibility__badge {
    font-size: var(--forge-font-size-xs);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-analysis-visibility__group-count {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-analysis-visibility__entry {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border-radius: var(--forge-radius-sm);
  }

  .forge-analysis-visibility__entry:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-analysis-visibility__name {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-sm);
    font-weight: 500;
    color: var(--forge-text-primary);
  }

  .forge-analysis-visibility__counts {
    display: flex;
    gap: var(--forge-space-md);
  }

  .forge-analysis-visibility__count-read,
  .forge-analysis-visibility__count-write {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-analysis-visibility__count-read {
    color: rgba(100, 180, 255, 0.8);
  }

  .forge-analysis-visibility__count-write {
    color: rgba(255, 160, 100, 0.8);
  }
</style>
