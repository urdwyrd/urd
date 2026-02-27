<script lang="ts">
  /**
   * StateInspector — shows current world state during playback.
   *
   * Uses InspectorPanel + InspectorSection to display location details
   * and entity properties. Subscribes to playback.state.changed.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { playbackService } from './_shared/PlaybackService.svelte';
  import type { PlaybackState, PlaybackEntity } from './_shared/playback-types';
  import InspectorPanel from '$lib/app/views/inspectors/_shared/InspectorPanel.svelte';
  import InspectorSection from '$lib/app/views/inspectors/_shared/InspectorSection.svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let playState: PlaybackState = $state(playbackService.state);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    unsubscribers.push(
      bus.subscribe('playback.state.changed', () => {
        playState = playbackService.state;
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function handleEntityClick(entity: PlaybackEntity): void {
    selectionContext.select([{ kind: 'entity', id: entity.id }]);
  }

  function formatValue(value: unknown): string {
    if (value === null || value === undefined) return '—';
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  }
</script>

<InspectorPanel
  title="State Inspector"
  emptyMessage="Start playback to inspect world state"
  hasContent={playState.status === 'playing'}
>
  {#if playState.currentLocation}
    <InspectorSection title="Current Location">
      <div class="forge-state-inspector__location">
        <div class="forge-state-inspector__loc-name">{playState.currentLocation.name}</div>
        {#if playState.currentLocation.description}
          <div class="forge-state-inspector__loc-desc">
            {playState.currentLocation.description.slice(0, 120)}{playState.currentLocation.description.length > 120 ? '...' : ''}
          </div>
        {/if}
        <div class="forge-state-inspector__loc-meta">
          {playState.currentLocation.exits.length} exit{playState.currentLocation.exits.length !== 1 ? 's' : ''}
          · Turn {playState.turnCount}
        </div>
      </div>
    </InspectorSection>

    {#if playState.entities.length > 0}
      <InspectorSection title="Entities ({playState.entities.length})">
        {#each playState.entities as entity}
          <div
            class="forge-state-inspector__entity"
            role="button"
            tabindex="0"
            onclick={() => handleEntityClick(entity)}
            onkeydown={(e) => { if (e.key === 'Enter') handleEntityClick(entity); }}
          >
            <div class="forge-state-inspector__entity-header">
              {#if entity.type}
                <span class="forge-state-inspector__entity-type">{entity.type}</span>
              {/if}
              <span class="forge-state-inspector__entity-name">{entity.name}</span>
            </div>
            {#if Object.keys(entity.properties).length > 0}
              <dl class="forge-state-inspector__props">
                {#each Object.entries(entity.properties) as [key, value]}
                  <dt class="forge-state-inspector__prop-key">{key}</dt>
                  <dd class="forge-state-inspector__prop-value">{formatValue(value)}</dd>
                {/each}
              </dl>
            {/if}
          </div>
        {/each}
      </InspectorSection>
    {/if}

    {#if playState.choices.length > 0}
      <InspectorSection title="Active Choices" collapsed>
        <div class="forge-state-inspector__choices">
          {#each playState.choices as choice}
            <div class="forge-state-inspector__choice" class:forge-state-inspector__choice--consumed={!choice.available}>
              {choice.label}
              {#if !choice.available}
                <span class="forge-state-inspector__consumed-tag">consumed</span>
              {/if}
            </div>
          {/each}
        </div>
      </InspectorSection>
    {/if}
  {/if}
</InspectorPanel>

<style>
  .forge-state-inspector__location {
    margin-bottom: var(--forge-space-sm);
  }

  .forge-state-inspector__loc-name {
    font-weight: 600;
    color: var(--forge-text-primary);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-state-inspector__loc-desc {
    color: var(--forge-text-secondary);
    font-size: var(--forge-font-size-xs);
    line-height: 1.4;
    margin-bottom: var(--forge-space-xs);
  }

  .forge-state-inspector__loc-meta {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-state-inspector__entity {
    padding: var(--forge-space-sm);
    margin-bottom: var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    cursor: pointer;
  }

  .forge-state-inspector__entity:hover {
    background: var(--forge-bg-hover);
  }

  .forge-state-inspector__entity-header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-state-inspector__entity-type {
    padding: 1px 5px;
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-muted);
    font-size: 10px;
    text-transform: uppercase;
  }

  .forge-state-inspector__entity-name {
    font-weight: 600;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
  }

  .forge-state-inspector__props {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--forge-space-xs) var(--forge-space-md);
    margin: 0;
    padding-left: var(--forge-space-sm);
  }

  .forge-state-inspector__prop-key {
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-state-inspector__prop-value {
    color: var(--forge-text-primary);
    margin: 0;
    font-size: var(--forge-font-size-xs);
    word-break: break-word;
  }

  .forge-state-inspector__choices {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
  }

  .forge-state-inspector__choice {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
  }

  .forge-state-inspector__choice--consumed {
    opacity: 0.5;
    text-decoration: line-through;
  }

  .forge-state-inspector__consumed-tag {
    font-size: 10px;
    color: var(--forge-text-muted);
    margin-left: var(--forge-space-xs);
  }
</style>
