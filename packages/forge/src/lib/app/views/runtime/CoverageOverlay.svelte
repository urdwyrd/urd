<script lang="ts">
  /**
   * CoverageOverlay — shows coverage statistics from playback.
   *
   * Displays progress bars for locations, sections, choices, and exits
   * visited during the current playback session.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { playbackService } from './_shared/PlaybackService.svelte';
  import type { CoverageData, CoverageCategory } from './_shared/playback-types';
  import InspectorPanel from '$lib/app/views/inspectors/_shared/InspectorPanel.svelte';
  import InspectorSection from '$lib/app/views/inspectors/_shared/InspectorSection.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: { viewport: null };
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let coverageData: CoverageData = $state(playbackService.coverage);
  let isActive = $state(playbackService.state.status === 'playing');

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    unsubscribers.push(
      bus.subscribe('coverage.overlay.updated', () => {
        coverageData = playbackService.coverage;
        isActive = playbackService.state.status === 'playing';
      }),
    );
    unsubscribers.push(
      bus.subscribe('coverage.overlay.cleared', () => {
        coverageData = playbackService.coverage;
        isActive = false;
      }),
    );
    unsubscribers.push(
      bus.subscribe('playback.state.changed', () => {
        isActive = playbackService.state.status === 'playing';
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function pct(cat: CoverageCategory): number {
    if (cat.total === 0) return 0;
    return Math.round((cat.visited.length / cat.total) * 100);
  }

  function overallPct(): number {
    const totalItems =
      coverageData.locations.total +
      coverageData.sections.total +
      coverageData.choices.total +
      coverageData.exits.total;
    if (totalItems === 0) return 0;
    const visitedItems =
      coverageData.locations.visited.length +
      coverageData.sections.visited.length +
      coverageData.choices.visited.length +
      coverageData.exits.visited.length;
    return Math.round((visitedItems / totalItems) * 100);
  }

  const categories = [
    { key: 'locations' as const, label: 'Locations' },
    { key: 'sections' as const, label: 'Sections' },
    { key: 'choices' as const, label: 'Choices' },
    { key: 'exits' as const, label: 'Exits' },
  ];

  function findUnvisited(cat: CoverageCategory): string[] {
    // We only know visited items, not the full set of IDs.
    // For the mock implementation, we just display visited items.
    // When Wyrd is integrated, this will show the diff against all items.
    return [];
  }
</script>

<InspectorPanel
  title="Coverage"
  emptyMessage="Start playback to track coverage"
  hasContent={isActive}
>
  <div class="forge-coverage">
    <div class="forge-coverage__overall">
      <span class="forge-coverage__overall-label">Overall</span>
      <span class="forge-coverage__overall-pct">{overallPct()}%</span>
    </div>

    {#each categories as cat}
      {@const data = coverageData[cat.key]}
      {@const percent = pct(data)}
      <div class="forge-coverage__category">
        <div class="forge-coverage__cat-header">
          <span class="forge-coverage__cat-label">{cat.label}</span>
          <span class="forge-coverage__cat-count">
            {data.visited.length} / {data.total}
          </span>
        </div>
        <div class="forge-coverage__bar">
          <div
            class="forge-coverage__bar-fill"
            class:forge-coverage__bar-fill--low={percent < 50}
            style="width: {percent}%"
          ></div>
        </div>
      </div>
    {/each}

    {#each categories as cat}
      {@const data = coverageData[cat.key]}
      {@const unvisited = findUnvisited(data)}
      {#if unvisited.length > 0}
        <InspectorSection title="Unvisited {cat.label}" collapsed>
          <ul class="forge-coverage__unvisited-list">
            {#each unvisited as item}
              <li class="forge-coverage__unvisited-item">{item}</li>
            {/each}
          </ul>
        </InspectorSection>
      {/if}
    {/each}
  </div>
</InspectorPanel>

<style>
  .forge-coverage {
    font-family: var(--forge-font-family-ui);
  }

  .forge-coverage__overall {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--forge-space-lg);
    padding-bottom: var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-coverage__overall-label {
    font-size: var(--forge-font-size-sm);
    font-weight: 600;
    color: var(--forge-text-primary);
  }

  .forge-coverage__overall-pct {
    font-size: var(--forge-font-size-lg);
    font-weight: 700;
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-mono);
  }

  .forge-coverage__category {
    margin-bottom: var(--forge-space-md);
  }

  .forge-coverage__cat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--forge-space-xs);
  }

  .forge-coverage__cat-label {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
  }

  .forge-coverage__cat-count {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }

  .forge-coverage__bar {
    height: 6px;
    border-radius: 3px;
    background: var(--forge-runtime-coverage-bar-bg, rgba(255, 255, 255, 0.08));
    overflow: hidden;
  }

  .forge-coverage__bar-fill {
    height: 100%;
    border-radius: 3px;
    background: var(--forge-runtime-coverage-bar-fill, #4caf50);
    transition: width 0.3s ease;
  }

  .forge-coverage__bar-fill--low {
    background: var(--forge-runtime-coverage-bar-low, #e94560);
  }

  .forge-coverage__unvisited-list {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .forge-coverage__unvisited-item {
    padding: var(--forge-space-xs) 0;
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }
</style>
