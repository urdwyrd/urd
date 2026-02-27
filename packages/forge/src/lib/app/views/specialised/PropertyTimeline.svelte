<script lang="ts">
  /**
   * PropertyTimeline — timeline of property changes over playback turns.
   *
   * Subscribes to playback.event to track property-related events.
   * X-axis = turns, Y-axis = event list. Simple vertical timeline
   * with dots and labels.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { playbackService } from '../runtime/_shared/PlaybackService.svelte';
  import type { PlaybackEvent } from '../runtime/_shared/playback-types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface TimelineEntry {
    turn: number;
    type: string;
    summary: string;
    timestamp: number;
  }

  let entries: TimelineEntry[] = $state([]);
  let currentTurn = $state(0);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    // Catch up with existing events
    for (const event of playbackService.events) {
      addEvent(event);
    }
    currentTurn = playbackService.state.turnCount;

    unsubscribers.push(
      bus.subscribe('playback.event', (payload: Record<string, unknown>) => {
        const event = payload.event as PlaybackEvent;
        addEvent(event);
        currentTurn = playbackService.state.turnCount;
      }),
    );

    unsubscribers.push(
      bus.subscribe('playback.state.changed', () => {
        if (playbackService.state.status === 'idle') {
          entries = [];
          currentTurn = 0;
        }
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function addEvent(event: PlaybackEvent): void {
    // Only track meaningful state-change events
    if (event.type === 'world_loaded' || event.type === 'world_reset') {
      entries = [];
      return;
    }

    entries = [...entries, {
      turn: playbackService.state.turnCount,
      type: event.type,
      summary: event.summary,
      timestamp: event.timestamp,
    }];
  }

  function dotColour(type: string): string {
    switch (type) {
      case 'move': return 'var(--forge-runtime-event-move, #5b9bd5)';
      case 'choice_made': return 'var(--forge-runtime-event-dialogue, #e6a817)';
      case 'section_enter': return 'var(--forge-runtime-event-section, #9b59b6)';
      case 'set': return 'var(--forge-runtime-event-set, #4caf50)';
      case 'exhausted': return 'var(--forge-runtime-event-narration, #888)';
      default: return 'var(--forge-text-muted)';
    }
  }
</script>

<div class="forge-prop-timeline">
  <div class="forge-prop-timeline__toolbar">
    <span class="forge-prop-timeline__title">Property Timeline</span>
    <div class="forge-prop-timeline__spacer"></div>
    {#if currentTurn > 0}
      <span class="forge-prop-timeline__turn">Turn {currentTurn}</span>
    {/if}
  </div>

  {#if entries.length === 0}
    <div class="forge-prop-timeline__empty">
      <p>No events recorded yet</p>
      <p class="forge-prop-timeline__hint">Start playback to track property changes over time</p>
    </div>
  {:else}
    <div class="forge-prop-timeline__content">
      <div class="forge-prop-timeline__track">
        {#each entries as entry, i (i)}
          <div class="forge-prop-timeline__entry">
            <div class="forge-prop-timeline__connector">
              <div
                class="forge-prop-timeline__dot"
                style="background-color: {dotColour(entry.type)}"
              ></div>
              {#if i < entries.length - 1}
                <div class="forge-prop-timeline__line"></div>
              {/if}
            </div>
            <div class="forge-prop-timeline__detail">
              <div class="forge-prop-timeline__detail-header">
                <span class="forge-prop-timeline__event-type">{entry.type}</span>
                <span class="forge-prop-timeline__event-turn">T{entry.turn}</span>
              </div>
              <span class="forge-prop-timeline__event-summary">{entry.summary}</span>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .forge-prop-timeline {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-prop-timeline__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-prop-timeline__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-prop-timeline__spacer {
    flex: 1;
  }

  .forge-prop-timeline__turn {
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-prop-timeline__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-prop-timeline__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-prop-timeline__content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-md);
  }

  .forge-prop-timeline__track {
    display: flex;
    flex-direction: column;
  }

  .forge-prop-timeline__entry {
    display: flex;
    gap: var(--forge-space-md);
    min-height: 36px;
  }

  .forge-prop-timeline__connector {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 16px;
    flex-shrink: 0;
  }

  .forge-prop-timeline__dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 4px;
  }

  .forge-prop-timeline__line {
    width: 1px;
    flex: 1;
    background: var(--forge-border-zone);
    min-height: 12px;
  }

  .forge-prop-timeline__detail {
    flex: 1;
    padding-bottom: var(--forge-space-sm);
  }

  .forge-prop-timeline__detail-header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    margin-bottom: 2px;
  }

  .forge-prop-timeline__event-type {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--forge-text-secondary);
  }

  .forge-prop-timeline__event-turn {
    font-size: 10px;
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-prop-timeline__event-summary {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
    line-height: 1.4;
  }
</style>
