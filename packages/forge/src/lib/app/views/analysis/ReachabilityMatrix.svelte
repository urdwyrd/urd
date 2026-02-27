<script lang="ts">
  /**
   * ReachabilityMatrix — on-demand BFS reachability analysis for locations.
   *
   * Shows a "Compute Reachability" button when idle. On click, reads
   * urdJson.locations and factSet.exits, runs BFS from each location,
   * and displays an NxN reachability grid.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import type {
    ResolvedCompilerOutput,
    UrdLocation,
    ExitEdge,
  } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  type ReachabilityState = 'idle' | 'computing' | 'done';

  let computeState: ReachabilityState = $state('idle');
  let locations: string[] = $state([]);
  let matrix: boolean[][] = $state([]);
  let stale = $state(false);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    unsubscribers.push(bus.subscribe('compiler.completed', () => {
      if (computeState === 'done') stale = true;
    }));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function computeReachability(): void {
    computeState = 'computing';
    stale = false;

    // Read data directly from projections (urdJson and factSet are base chunks)
    const urdJson = projectionRegistry.get<{ locations: UrdLocation[] }>('urd.projection.urdJson');
    const factSetData = projectionRegistry.get<{ exits: ExitEdge[] }>('urd.projection.factSet');

    if (!urdJson || !factSetData) {
      computeState = 'idle';
      return;
    }

    const locs = urdJson.locations ?? [];
    const locNames = locs.map((l) => l.name || l.id);
    const locSet = new Set(locNames);

    // Build adjacency list from exits
    const adjacency = new Map<string, Set<string>>();
    for (const name of locNames) {
      adjacency.set(name, new Set());
    }

    for (const exit of (factSetData.exits ?? [])) {
      if (locSet.has(exit.from_location) && locSet.has(exit.to_location)) {
        adjacency.get(exit.from_location)!.add(exit.to_location);
      }
    }

    // BFS from each location
    const reachability: boolean[][] = [];

    for (let i = 0; i < locNames.length; i++) {
      const row: boolean[] = new Array(locNames.length).fill(false);
      const visited = new Set<string>();
      const queue: string[] = [locNames[i]];
      visited.add(locNames[i]);

      while (queue.length > 0) {
        const current = queue.shift()!;
        const neighbours = adjacency.get(current);
        if (!neighbours) continue;

        for (const next of neighbours) {
          if (!visited.has(next)) {
            visited.add(next);
            queue.push(next);
          }
        }
      }

      for (let j = 0; j < locNames.length; j++) {
        row[j] = visited.has(locNames[j]);
      }
      reachability.push(row);
    }

    locations = locNames;
    matrix = reachability;
    computeState = 'done';
  }

  let reachableCount = $derived.by(() => {
    let count = 0;
    for (const row of matrix) {
      for (const cell of row) {
        if (cell) count++;
      }
    }
    return count;
  });

  let totalCells = $derived(locations.length * locations.length);
</script>

<div class="forge-analysis-reachability">
  {#if computeState === 'idle'}
    <div class="forge-analysis-reachability__prompt">
      <p class="forge-analysis-reachability__description">
        Compute a full reachability matrix across all locations using BFS traversal.
      </p>
      <button
        class="forge-analysis-reachability__compute-btn"
        onclick={computeReachability}
      >
        Compute Reachability
      </button>
    </div>
  {:else if computeState === 'computing'}
    <div class="forge-analysis-reachability__computing">
      Computing reachability...
    </div>
  {:else}
    <div class="forge-analysis-reachability__results">
      {#if stale}
        <div class="forge-analysis-reachability__stale-banner">
          Data may be stale.
          <button
            class="forge-analysis-reachability__recompute-btn"
            onclick={computeReachability}
          >
            Recompute
          </button>
        </div>
      {/if}

      <div class="forge-analysis-reachability__summary">
        {reachableCount} / {totalCells} cells reachable ({locations.length} locations)
      </div>

      {#if locations.length === 0}
        <div class="forge-analysis-reachability__empty">
          No locations found in the world
        </div>
      {:else}
        <div class="forge-analysis-reachability__matrix-wrapper">
          <table class="forge-analysis-reachability__table">
            <thead>
              <tr>
                <th class="forge-analysis-reachability__corner"></th>
                {#each locations as loc}
                  <th class="forge-analysis-reachability__col-header" title={loc}>
                    {loc.length > 8 ? loc.slice(0, 7) + '...' : loc}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each locations as fromLoc, i}
                <tr>
                  <td class="forge-analysis-reachability__row-header" title={fromLoc}>
                    {fromLoc.length > 10 ? fromLoc.slice(0, 9) + '...' : fromLoc}
                  </td>
                  {#each locations as _toLoc, j}
                    <td
                      class="forge-analysis-reachability__cell"
                      class:forge-analysis-reachability__cell--reachable={matrix[i][j]}
                      class:forge-analysis-reachability__cell--self={i === j}
                      class:forge-analysis-reachability__cell--unreachable={!matrix[i][j] && i !== j}
                    ></td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-analysis-reachability {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-analysis-reachability__prompt {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    gap: var(--forge-space-lg);
    padding: var(--forge-space-xl);
  }

  .forge-analysis-reachability__description {
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
    text-align: center;
    max-width: 400px;
  }

  .forge-analysis-reachability__compute-btn,
  .forge-analysis-reachability__recompute-btn {
    padding: var(--forge-space-sm) var(--forge-space-lg);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
  }

  .forge-analysis-reachability__compute-btn:hover,
  .forge-analysis-reachability__recompute-btn:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-analysis-reachability__computing {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-reachability__results {
    display: flex;
    flex-direction: column;
    padding: var(--forge-space-md);
    gap: var(--forge-space-md);
  }

  .forge-analysis-reachability__stale-banner {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-sm) var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border: 1px solid var(--forge-status-warning);
    border-radius: var(--forge-radius-sm);
    color: var(--forge-status-warning);
    font-size: var(--forge-font-size-sm);
  }

  .forge-analysis-reachability__summary {
    font-size: var(--forge-font-size-sm);
    color: var(--forge-text-secondary);
  }

  .forge-analysis-reachability__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--forge-space-xl);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }

  .forge-analysis-reachability__matrix-wrapper {
    overflow: auto;
  }

  .forge-analysis-reachability__table {
    border-collapse: collapse;
    font-size: var(--forge-font-size-xs);
  }

  .forge-analysis-reachability__corner {
    width: 80px;
    min-width: 80px;
  }

  .forge-analysis-reachability__col-header {
    writing-mode: vertical-lr;
    text-orientation: mixed;
    padding: var(--forge-space-xs);
    color: var(--forge-text-secondary);
    font-weight: 500;
    font-family: var(--forge-font-family-mono);
    max-width: 28px;
    overflow: hidden;
  }

  .forge-analysis-reachability__row-header {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    color: var(--forge-text-secondary);
    font-weight: 500;
    font-family: var(--forge-font-family-mono);
    text-align: right;
    white-space: nowrap;
  }

  .forge-analysis-reachability__cell {
    width: 24px;
    height: 24px;
    min-width: 24px;
    min-height: 24px;
    border: 1px solid var(--forge-border-zone);
  }

  .forge-analysis-reachability__cell--reachable {
    background-color: var(--forge-status-success);
    opacity: 0.6;
  }

  .forge-analysis-reachability__cell--self {
    background-color: var(--forge-text-muted);
    opacity: 0.3;
  }

  .forge-analysis-reachability__cell--unreachable {
    background-color: transparent;
  }
</style>
