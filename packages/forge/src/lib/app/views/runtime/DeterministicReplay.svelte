<script lang="ts">
  /**
   * DeterministicReplay — record and replay playback sessions.
   *
   * Subscribes to playback.event to capture actions during recording.
   * Replay mode steps through captured events one by one, allowing
   * deterministic reproduction of playback sessions.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { playbackService } from './_shared/PlaybackService.svelte';
  import type { PlaybackEvent } from './_shared/playback-types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  type ReplayStatus = 'idle' | 'recording' | 'stopped' | 'replaying';

  interface RecordedAction {
    index: number;
    type: string;
    summary: string;
    timestamp: number;
    relativeMs: number;
    details?: Record<string, unknown>;
  }

  let status: ReplayStatus = $state('idle');
  let recordedActions: RecordedAction[] = $state([]);
  let replayPosition = $state(-1);
  let recordStartTime = $state(0);
  let scrollContainer: HTMLDivElement | undefined = $state(undefined);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    unsubscribers.push(
      bus.subscribe('playback.event', (payload: Record<string, unknown>) => {
        if (status !== 'recording') return;
        const event = payload.event as PlaybackEvent;
        // Skip system events from recording
        if (event.type === 'world_loaded' || event.type === 'world_reset') return;

        const action: RecordedAction = {
          index: recordedActions.length,
          type: event.type,
          summary: event.summary,
          timestamp: event.timestamp,
          relativeMs: event.timestamp - recordStartTime,
          details: event.details,
        };
        recordedActions = [...recordedActions, action];
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function handleRecord(): void {
    if (status === 'recording') return;
    recordedActions = [];
    replayPosition = -1;
    recordStartTime = Date.now();
    status = 'recording';
  }

  function handleStop(): void {
    if (status === 'recording') {
      status = 'stopped';
    } else if (status === 'replaying') {
      status = 'stopped';
    }
  }

  function handlePlay(): void {
    if (recordedActions.length === 0) return;
    replayPosition = 0;
    status = 'replaying';
    // Load the world fresh for replay
    playbackService.loadWorld();
  }

  function handleStep(): void {
    if (recordedActions.length === 0) return;
    if (status !== 'replaying' && status !== 'stopped') return;
    if (replayPosition >= recordedActions.length - 1) return;

    status = 'replaying';
    replayPosition++;

    const action = recordedActions[replayPosition];
    replayAction(action);
  }

  function replayAction(action: RecordedAction): void {
    switch (action.type) {
      case 'move': {
        const direction = action.details?.direction as string | undefined;
        if (direction) playbackService.move(direction);
        break;
      }
      case 'choice_made': {
        const choiceId = action.details?.choiceId as string | undefined;
        if (choiceId) playbackService.choose(choiceId);
        break;
      }
      case 'section_enter': {
        const sectionId = action.details?.sectionId as string | undefined;
        if (sectionId) playbackService.enterSection(sectionId);
        break;
      }
    }
  }

  function handleClear(): void {
    recordedActions = [];
    replayPosition = -1;
    status = 'idle';
  }

  function formatTime(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainder = seconds % 60;
    const millis = ms % 1000;
    if (minutes > 0) {
      return `${minutes}:${String(remainder).padStart(2, '0')}.${String(millis).padStart(3, '0')}`;
    }
    return `${remainder}.${String(millis).padStart(3, '0')}s`;
  }

  function badgeClass(type: string): string {
    return `forge-replay__badge forge-replay__badge--${type}`;
  }
</script>

<div class="forge-replay">
  <div class="forge-replay__toolbar">
    <span class="forge-replay__title">Deterministic Replay</span>
    <div class="forge-replay__spacer"></div>

    {#if status === 'idle' || status === 'stopped'}
      <button
        class="forge-replay__btn forge-replay__btn--record"
        onclick={handleRecord}
        title="Start recording"
      >
        ◉ Record
      </button>
    {/if}

    {#if status === 'recording'}
      <span class="forge-replay__status forge-replay__status--recording">◉ Recording</span>
      <button class="forge-replay__btn" onclick={handleStop} title="Stop recording">
        ◼ Stop
      </button>
    {/if}

    {#if status === 'stopped' && recordedActions.length > 0}
      <button
        class="forge-replay__btn forge-replay__btn--play"
        onclick={handlePlay}
        title="Replay from start"
      >
        ▸ Play
      </button>
      <button class="forge-replay__btn" onclick={handleStep} title="Step forward">
        ▸▸ Step
      </button>
    {/if}

    {#if status === 'replaying'}
      <span class="forge-replay__status forge-replay__status--replaying">▸ Replaying</span>
      <button class="forge-replay__btn" onclick={handleStep} title="Step forward">
        ▸▸ Step
      </button>
      <button class="forge-replay__btn" onclick={handleStop} title="Pause">
        ◼ Pause
      </button>
    {/if}

    {#if recordedActions.length > 0}
      <button class="forge-replay__btn" onclick={handleClear} title="Clear recording">
        Clear
      </button>
    {/if}
  </div>

  {#if status === 'replaying' || status === 'stopped'}
    {#if recordedActions.length > 0}
      <div class="forge-replay__position-bar">
        <span class="forge-replay__position-label">
          Step {replayPosition + 1} / {recordedActions.length}
        </span>
      </div>
    {/if}
  {/if}

  {#if recordedActions.length === 0}
    <div class="forge-replay__empty">
      <p>Record a playback session to create a replay</p>
      <p class="forge-replay__hint">
        Click Record, interact with the Play Panel, then Stop to capture actions
      </p>
    </div>
  {:else}
    <div class="forge-replay__list" bind:this={scrollContainer}>
      {#each recordedActions as action (action.index)}
        <div
          class="forge-replay__row"
          class:forge-replay__row--current={action.index === replayPosition}
          class:forge-replay__row--past={action.index < replayPosition}
        >
          <span class="forge-replay__index">{action.index + 1}</span>
          <span class={badgeClass(action.type)}>{action.type}</span>
          <span class="forge-replay__summary">{action.summary}</span>
          <span class="forge-replay__time">{formatTime(action.relativeMs)}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .forge-replay {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-replay__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 32px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-replay__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-replay__spacer {
    flex: 1;
  }

  .forge-replay__btn {
    padding: 2px 8px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    cursor: pointer;
  }

  .forge-replay__btn:hover {
    background: var(--forge-bg-hover);
  }

  .forge-replay__btn--record {
    color: var(--forge-graph-node-unreachable, #e94560);
    border-color: var(--forge-graph-node-unreachable, #e94560);
  }

  .forge-replay__btn--play {
    color: var(--forge-runtime-play-active, #4caf50);
    border-color: var(--forge-runtime-play-active, #4caf50);
  }

  .forge-replay__status {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
  }

  .forge-replay__status--recording {
    color: var(--forge-graph-node-unreachable, #e94560);
  }

  .forge-replay__status--replaying {
    color: var(--forge-runtime-play-active, #4caf50);
  }

  .forge-replay__position-bar {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 22px;
    background: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-replay__position-label {
    font-size: var(--forge-font-size-xs);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-replay__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-replay__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-replay__list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .forge-replay__row {
    display: flex;
    align-items: baseline;
    gap: var(--forge-space-sm);
    padding: var(--forge-space-xs) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-subtle, rgba(255, 255, 255, 0.04));
  }

  .forge-replay__row:hover {
    background: var(--forge-bg-hover);
  }

  .forge-replay__row--current {
    background: rgba(76, 175, 80, 0.12);
    border-left: 2px solid var(--forge-runtime-play-active, #4caf50);
  }

  .forge-replay__row--past {
    opacity: 0.5;
  }

  .forge-replay__index {
    width: 24px;
    flex-shrink: 0;
    text-align: right;
    font-size: 10px;
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }

  .forge-replay__badge {
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

  .forge-replay__badge--move {
    background: var(--forge-runtime-event-move, #5b9bd5);
    color: #fff;
  }

  .forge-replay__badge--choice_made {
    background: var(--forge-runtime-event-dialogue, #e6a817);
    color: #fff;
  }

  .forge-replay__badge--section_enter {
    background: var(--forge-runtime-event-section, #9b59b6);
    color: #fff;
  }

  .forge-replay__badge--set {
    background: var(--forge-runtime-event-set, #4caf50);
    color: #fff;
  }

  .forge-replay__badge--exhausted {
    background: var(--forge-runtime-event-narration, #888);
    color: #fff;
  }

  .forge-replay__summary {
    flex: 1;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-replay__time {
    flex-shrink: 0;
    font-size: 10px;
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-muted);
  }
</style>
