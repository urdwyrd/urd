<script lang="ts">
  /**
   * EventLog — scrolling chronological log of playback events.
   *
   * Subscribes to playback.event bus channel. Each event is displayed
   * as a row with a type badge, summary text, and turn number.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { playbackService } from './_shared/PlaybackService.svelte';
  import type { PlaybackEvent } from './_shared/playback-types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: { viewport: null };
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let localEvents: PlaybackEvent[] = $state([]);
  let autoScroll = $state(true);
  let scrollContainer: HTMLDivElement | undefined = $state(undefined);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    // Catch up with any existing events
    localEvents = [...playbackService.events];

    unsubscribers.push(
      bus.subscribe('playback.event', async () => {
        localEvents = [...playbackService.events];
        if (autoScroll) {
          await tick();
          if (scrollContainer) {
            scrollContainer.scrollTop = scrollContainer.scrollHeight;
          }
        }
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function handleClear(): void {
    localEvents = [];
  }

  function handleScroll(): void {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    // Disable auto-scroll if user scrolled up more than 20px from bottom
    autoScroll = scrollHeight - scrollTop - clientHeight < 20;
  }

  function badgeClass(type: string): string {
    return `forge-event-log__badge forge-event-log__badge--${type}`;
  }
</script>

<div class="forge-event-log">
  <div class="forge-event-log__toolbar">
    <span class="forge-event-log__title">Event Log</span>
    <span class="forge-event-log__count">{localEvents.length}</span>
    <div class="forge-event-log__spacer"></div>
    <button class="forge-event-log__btn" onclick={handleClear} title="Clear log">
      Clear
    </button>
  </div>

  {#if localEvents.length === 0}
    <div class="forge-event-log__empty">
      <p>No events yet</p>
      <p class="forge-event-log__hint">Start playback to see events</p>
    </div>
  {:else}
    <div
      class="forge-event-log__list"
      bind:this={scrollContainer}
      onscroll={handleScroll}
    >
      {#each localEvents as event (event.id)}
        <div class="forge-event-log__row">
          <span class={badgeClass(event.type)}>{event.type}</span>
          <span class="forge-event-log__summary">{event.summary}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-event-log {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-event-log__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-event-log__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-event-log__count {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }

  .forge-event-log__spacer {
    flex: 1;
  }

  .forge-event-log__btn {
    padding: 1px 8px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-event-log__btn:hover {
    background: var(--forge-bg-hover);
  }

  .forge-event-log__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-event-log__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-event-log__list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .forge-event-log__row {
    display: flex;
    align-items: baseline;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-xs) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-subtle, rgba(255, 255, 255, 0.04));
  }

  .forge-event-log__row:hover {
    background: var(--forge-bg-hover);
  }

  .forge-event-log__badge {
    display: inline-block;
    padding: 0 5px;
    border-radius: var(--forge-radius-sm);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    flex-shrink: 0;
    line-height: 18px;
  }

  .forge-event-log__badge--move {
    background: var(--forge-runtime-event-move, #5b9bd5);
    color: #fff;
  }

  .forge-event-log__badge--dialogue {
    background: var(--forge-runtime-event-dialogue, #e6a817);
    color: #fff;
  }

  .forge-event-log__badge--set {
    background: var(--forge-runtime-event-set, #4caf50);
    color: #fff;
  }

  .forge-event-log__badge--narration {
    background: var(--forge-runtime-event-narration, #888);
    color: #fff;
  }

  .forge-event-log__badge--section_enter {
    background: var(--forge-runtime-event-section, #9b59b6);
    color: #fff;
  }

  .forge-event-log__badge--choice_made {
    background: var(--forge-runtime-event-dialogue, #e6a817);
    color: #fff;
  }

  .forge-event-log__badge--exhausted {
    background: var(--forge-runtime-event-narration, #888);
    color: #fff;
  }

  .forge-event-log__badge--world_loaded,
  .forge-event-log__badge--world_reset {
    background: var(--forge-runtime-play-active, #4caf50);
    color: #fff;
  }

  .forge-event-log__summary {
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-xs);
  }
</style>
