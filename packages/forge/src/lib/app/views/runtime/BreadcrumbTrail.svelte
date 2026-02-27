<script lang="ts">
  /**
   * BreadcrumbTrail — horizontal trail of visited locations during playback.
   *
   * Shows the path taken: Location A → Location B → Location C.
   * Current location is highlighted. Clicking selects the location.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { playbackService } from './_shared/PlaybackService.svelte';
  import type { PlaybackState } from './_shared/playback-types';
  import type { UrdWorld } from '$lib/app/compiler/types';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: { viewport: null };
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let playState: PlaybackState = $state(playbackService.state);
  let locationNames: Map<string, string> = $state(new Map());

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshLocationNames();
    unsubscribers.push(
      bus.subscribe('playback.state.changed', () => {
        playState = playbackService.state;
      }),
    );
    unsubscribers.push(bus.subscribe('compiler.completed', refreshLocationNames));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshLocationNames(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    if (!urdJson) return;
    const map = new Map<string, string>();
    for (const loc of urdJson.locations) {
      map.set(loc.id, loc.name || loc.id);
    }
    locationNames = map;
  }

  function handleClick(locationId: string): void {
    selectionContext.select([{ kind: 'location', id: locationId }]);
  }

  function getName(locationId: string): string {
    return locationNames.get(locationId) ?? locationId;
  }
</script>

<div class="forge-breadcrumb-trail">
  <div class="forge-breadcrumb-trail__header">
    <span class="forge-breadcrumb-trail__title">Path</span>
    {#if playState.visitedLocations.length > 0}
      <span class="forge-breadcrumb-trail__count">
        {playState.visitedLocations.length} step{playState.visitedLocations.length !== 1 ? 's' : ''}
      </span>
    {/if}
  </div>

  {#if playState.status !== 'playing' || playState.visitedLocations.length === 0}
    <div class="forge-breadcrumb-trail__empty">
      <p>Start playback to see your path</p>
    </div>
  {:else}
    <div class="forge-breadcrumb-trail__trail">
      {#each playState.visitedLocations as locId, i}
        {#if i > 0}
          <span class="forge-breadcrumb-trail__separator">→</span>
        {/if}
        <button
          class="forge-breadcrumb-trail__crumb"
          class:forge-breadcrumb-trail__crumb--current={i === playState.visitedLocations.length - 1}
          onclick={() => handleClick(locId)}
          title={locId}
        >
          {getName(locId)}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-breadcrumb-trail {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-breadcrumb-trail__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-breadcrumb-trail__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-breadcrumb-trail__count {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }

  .forge-breadcrumb-trail__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
  }

  .forge-breadcrumb-trail__trail {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-md);
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    align-content: flex-start;
  }

  .forge-breadcrumb-trail__separator {
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    user-select: none;
  }

  .forge-breadcrumb-trail__crumb {
    display: inline-block;
    padding: 2px 8px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-breadcrumb-trail__crumb:hover {
    background: var(--forge-bg-hover);
    color: var(--forge-text-primary);
  }

  .forge-breadcrumb-trail__crumb--current {
    background: var(--forge-accent-primary);
    border-color: var(--forge-accent-primary);
    color: #fff;
  }

  .forge-breadcrumb-trail__crumb--current:hover {
    background: var(--forge-accent-primary);
    color: #fff;
  }
</style>
