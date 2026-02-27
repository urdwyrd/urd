<script lang="ts">
  /**
   * PlayPanel — primary interaction view for the mock runtime.
   *
   * Shows current location, available exits, dialogue choices, and
   * play/reset controls. Consumes PlaybackService for state.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { playbackService } from './_shared/PlaybackService.svelte';
  import type { PlaybackState } from './_shared/playback-types';
  import type { UrdWorld } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: { viewport: null };
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let playState: PlaybackState = $state(playbackService.state);
  let hasWorld = $state(false);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    checkWorld();
    unsubscribers.push(bus.subscribe('compiler.completed', checkWorld));
    unsubscribers.push(
      bus.subscribe('playback.state.changed', () => {
        playState = playbackService.state;
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function checkWorld(): void {
    const urdJson = projectionRegistry.get<UrdWorld>('urd.projection.urdJson');
    hasWorld = !!(urdJson && urdJson.locations.length > 0);
  }

  function handlePlay(): void {
    playbackService.loadWorld();
  }

  function handleReset(): void {
    playbackService.reset();
  }

  function handleMove(direction: string): void {
    playbackService.move(direction);
  }

  function handleChoose(choiceId: string): void {
    playbackService.choose(choiceId);
  }

  function handleSectionClick(sectionId: string): void {
    playbackService.enterSection(sectionId);
  }
</script>

<div class="forge-play-panel">
  <div class="forge-play-panel__toolbar">
    {#if playState.status === 'idle'}
      <button
        class="forge-play-panel__btn forge-play-panel__btn--play"
        onclick={handlePlay}
        disabled={!hasWorld}
        title={hasWorld ? 'Start playback' : 'No world to play — compile first'}
      >
        ▸ Play
      </button>
    {:else}
      <span class="forge-play-panel__status forge-play-panel__status--{playState.status}">
        {playState.status === 'playing' ? '▸ Playing' : '! Error'}
      </span>
      <button class="forge-play-panel__btn" onclick={handleReset} title="Reset to initial state">
        ↺ Reset
      </button>
    {/if}
    {#if playState.status === 'playing'}
      <span class="forge-play-panel__turn">Turn {playState.turnCount}</span>
    {/if}
  </div>

  {#if playState.status === 'idle'}
    <div class="forge-play-panel__empty">
      <p>Open a project and click Play to begin</p>
      <p class="forge-play-panel__hint">The runtime loads the compiled world and lets you explore</p>
    </div>
  {:else if playState.status === 'error'}
    <div class="forge-play-panel__empty">
      <p>Could not load the world</p>
      <p class="forge-play-panel__hint">Check that the project compiles successfully</p>
    </div>
  {:else if playState.currentLocation}
    <div class="forge-play-panel__content">
      <div class="forge-play-panel__location">
        <h3 class="forge-play-panel__location-name">{playState.currentLocation.name}</h3>
        {#if playState.currentLocation.description}
          <p class="forge-play-panel__location-desc">{playState.currentLocation.description}</p>
        {/if}
      </div>

      {#if playState.availableExits.length > 0}
        <div class="forge-play-panel__section">
          <div class="forge-play-panel__section-header">Exits</div>
          <div class="forge-play-panel__exits">
            {#each playState.availableExits as exit}
              <button
                class="forge-play-panel__exit-btn"
                onclick={() => handleMove(exit.direction)}
                title="Go to {exit.targetName}"
              >
                {exit.direction} → {exit.targetName}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      {#if playState.activeSection}
        <div class="forge-play-panel__section">
          <div class="forge-play-panel__section-header">
            Dialogue: {playState.activeSection}
          </div>
          <div class="forge-play-panel__choices">
            {#each playState.choices as choice}
              <button
                class="forge-play-panel__choice-btn"
                class:forge-play-panel__choice-btn--disabled={!choice.available}
                onclick={() => handleChoose(choice.id)}
                disabled={!choice.available}
              >
                {choice.label}
                {#if !choice.available}
                  <span class="forge-play-panel__choice-consumed">(consumed)</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      {#if playState.entities.length > 0}
        <div class="forge-play-panel__section">
          <div class="forge-play-panel__section-header">
            Entities here ({playState.entities.length})
          </div>
          <div class="forge-play-panel__entities">
            {#each playState.entities as entity}
              <span class="forge-play-panel__entity-tag">
                {entity.name}
                {#if entity.type}
                  <span class="forge-play-panel__entity-type">({entity.type})</span>
                {/if}
              </span>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-play-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-play-panel__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 32px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-play-panel__btn {
    padding: 2px 10px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-play-panel__btn:hover:not(:disabled) {
    background: var(--forge-bg-hover);
  }

  .forge-play-panel__btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .forge-play-panel__btn--play {
    color: var(--forge-runtime-play-active, #4caf50);
    border-color: var(--forge-runtime-play-active, #4caf50);
  }

  .forge-play-panel__status {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
  }

  .forge-play-panel__status--playing {
    color: var(--forge-runtime-play-active, #4caf50);
  }

  .forge-play-panel__status--error {
    color: var(--forge-graph-node-unreachable, #e94560);
  }

  .forge-play-panel__turn {
    margin-left: auto;
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }

  .forge-play-panel__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-play-panel__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-play-panel__content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-md);
  }

  .forge-play-panel__location {
    margin-bottom: var(--forge-space-lg);
  }

  .forge-play-panel__location-name {
    margin: 0 0 var(--forge-space-sm);
    font-size: var(--forge-font-size-lg);
    font-weight: 600;
    color: var(--forge-runtime-location-title, var(--forge-text-primary));
  }

  .forge-play-panel__location-desc {
    margin: 0;
    color: var(--forge-text-secondary);
    line-height: 1.5;
    font-family: var(--forge-font-family-body, var(--forge-font-family-ui));
  }

  .forge-play-panel__section {
    margin-bottom: var(--forge-space-lg);
  }

  .forge-play-panel__section-header {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--forge-space-sm);
  }

  .forge-play-panel__exits {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
  }

  .forge-play-panel__exit-btn {
    display: block;
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-md);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-runtime-exit-button, var(--forge-accent-primary));
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
    text-align: left;
  }

  .forge-play-panel__exit-btn:hover {
    background: var(--forge-bg-hover);
  }

  .forge-play-panel__choices {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
  }

  .forge-play-panel__choice-btn {
    display: block;
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-md);
    border: 1px solid var(--forge-runtime-event-dialogue, #e6a817);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
    text-align: left;
  }

  .forge-play-panel__choice-btn:hover:not(:disabled) {
    background: var(--forge-bg-hover);
  }

  .forge-play-panel__choice-btn--disabled {
    opacity: 0.4;
    cursor: default;
    border-color: var(--forge-border-zone);
  }

  .forge-play-panel__choice-consumed {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    margin-left: var(--forge-space-sm);
  }

  .forge-play-panel__entities {
    display: flex;
    flex-wrap: wrap;
    gap: var(--forge-space-xs);
  }

  .forge-play-panel__entity-tag {
    display: inline-block;
    padding: 2px 8px;
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-secondary);
    font-size: var(--forge-font-size-xs);
  }

  .forge-play-panel__entity-type {
    color: var(--forge-text-muted);
  }
</style>
