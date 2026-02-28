<script lang="ts">
  /**
   * PlayPanel — interactive fiction runtime panel.
   *
   * Layout: location header → scrolling narrative → choice groups → inventory bar.
   * Matches the URD Player reference design with left-border-accented narrative
   * entries, condition tags on disabled choices, and trait-based entity tags.
   */

  import { onMount, onDestroy, tick } from 'svelte';
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
  let autoScroll = $state(true);
  let viewport: HTMLDivElement | undefined = $state(undefined);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    checkWorld();
    unsubscribers.push(bus.subscribe('compiler.completed', checkWorld));
    unsubscribers.push(
      bus.subscribe('playback.state.changed', async () => {
        playState = playbackService.state;
        if (autoScroll) {
          await tick();
          scrollBottom();
        }
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

  function scrollBottom(): void {
    if (viewport) viewport.scrollTop = viewport.scrollHeight;
  }

  function handleScroll(): void {
    if (!viewport) return;
    const { scrollTop, scrollHeight, clientHeight } = viewport;
    autoScroll = scrollHeight - scrollTop - clientHeight < 30;
  }

  function fmtLoc(id: string): string {
    return id.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }
</script>

<div class="urd-play">
  {#if playState.status === 'idle'}
    <!-- Load screen -->
    <div class="urd-play__load">
      <h2 class="urd-play__load-title">URD Player</h2>
      <p class="urd-play__load-hint">Open a project and click Play to begin</p>
      <button
        class="urd-play__load-btn"
        onclick={() => playbackService.loadWorld()}
        disabled={!hasWorld}
      >
        {hasWorld ? 'Play' : 'No world — compile first'}
      </button>
    </div>
  {:else if playState.status === 'error'}
    <div class="urd-play__load">
      <h2 class="urd-play__load-title">Error</h2>
      <p class="urd-play__load-hint">Could not load the world. Check that the project compiles successfully.</p>
      <button class="urd-play__load-btn" onclick={() => playbackService.loadWorld()}>Retry</button>
    </div>
  {:else}
    <!-- Header bar -->
    <div class="urd-play__header">
      <span class="urd-play__header-title">{playState.currentLocation?.name ?? 'URD Player'}</span>
      <span class="urd-play__header-badge">Turn {playState.turnCount}</span>
      {#if playState.currentPhase}
        <span class="urd-play__header-badge urd-play__header-badge--phase">{playState.currentPhase}</span>
      {/if}
      <div class="urd-play__header-actions">
        <button class="urd-play__hdr-btn" onclick={() => playbackService.reset()}>Restart</button>
      </div>
    </div>

    {#if playState.gameOver}
      <div class="urd-play__game-over">
        <h2 class="urd-play__game-over-title">The End</h2>
        <button class="urd-play__load-btn" onclick={() => playbackService.reset()}>Play Again</button>
      </div>
    {/if}

    <!-- Game viewport: location header + narrative -->
    <div class="urd-play__viewport" bind:this={viewport} onscroll={handleScroll}>
      {#if playState.currentLocation}
        <div class="urd-play__loc-header">
          <div class="urd-play__loc-name">{playState.currentLocation.name}</div>
          {#if playState.currentLocation.description}
            <div class="urd-play__loc-desc">{playState.currentLocation.description}</div>
          {/if}
          {#if playState.entities.length > 0}
            <div class="urd-play__loc-contents">
              {#each playState.entities as ent}
                <span class="urd-play__loc-tag urd-play__loc-tag--{ent.traitClass}">@{ent.id}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Narrative log -->
      <div class="urd-play__narrative">
        {#each playState.narrative as line (line.id)}
          {#if line.kind === 'dlg_prompt'}
            <div class="urd-play__entry urd-play__entry--dlg-prompt">
              {#if line.speaker}
                <div class="urd-play__speaker">{line.speaker}</div>
              {/if}
              <div class="urd-play__text">{line.text}</div>
            </div>
          {:else if line.kind === 'dlg_response'}
            <div class="urd-play__entry urd-play__entry--dlg-response">
              {#if line.speaker}
                <div class="urd-play__speaker">{line.speaker}</div>
              {/if}
              <div class="urd-play__text">{line.text}</div>
            </div>
          {:else if line.kind === 'player_action'}
            <div class="urd-play__entry urd-play__entry--player-action">
              <div class="urd-play__speaker">You</div>
              <div class="urd-play__text">{line.text}</div>
            </div>
          {:else if line.kind === 'narration'}
            <div class="urd-play__entry urd-play__entry--narration">
              <div class="urd-play__text">{line.text}</div>
            </div>
          {:else if line.kind === 'blocked'}
            <div class="urd-play__entry urd-play__entry--blocked">
              <div class="urd-play__text">{line.text}</div>
            </div>
          {:else if line.kind === 'system'}
            <div class="urd-play__entry urd-play__entry--system">{line.text}</div>
          {/if}
        {/each}
      </div>
    </div>

    <!-- Choices area -->
    <div class="urd-play__choices-area">
      <!-- Dialogue choices -->
      {#if playState.inDialogue && playState.dialogueChoices.length > 0}
        <div class="urd-play__group-label">Dialogue</div>
        <div class="urd-play__choice-group">
          {#each playState.dialogueChoices as ch}
            <button
              class="urd-play__choice-btn"
              class:urd-play__choice-btn--disabled={!ch.available}
              onclick={() => ch.available && playbackService.chooseDialogue(ch.index)}
              disabled={!ch.available}
            >
              {ch.label}
              {#if ch.conditionText}
                <span class="urd-play__cond-tag" class:urd-play__cond-tag--ok={ch.available}>{ch.conditionText}</span>
              {/if}
            </button>
          {/each}
        </div>
        <button class="urd-play__exit-btn" onclick={() => playbackService.endDialogue()}>
          End conversation
        </button>
      {/if}

      <!-- Actions -->
      {#if playState.availableActions.length > 0}
        <div class="urd-play__group-label">Actions</div>
        <div class="urd-play__choice-group">
          {#each playState.availableActions as action}
            <button
              class="urd-play__choice-btn"
              class:urd-play__choice-btn--disabled={!action.available}
              onclick={() => action.available && playbackService.performAction(action.id)}
              disabled={!action.available}
            >
              {action.description ?? action.id}
              {#if action.targetName}
                <span class="urd-play__action-target">({action.targetName})</span>
              {/if}
              {#if action.conditionText}
                <span class="urd-play__cond-tag">{action.conditionText}</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}

      <!-- Exits -->
      {#if playState.availableExits.length > 0}
        <div class="urd-play__group-label">Exits</div>
        <div class="urd-play__choice-group">
          {#each playState.availableExits as exit}
            <button
              class="urd-play__exit-btn"
              class:urd-play__exit-btn--blocked={exit.blocked}
              onclick={() => exit.blocked ? playbackService.tryBlockedExit(exit.direction) : playbackService.moveTo(exit.targetId)}
            >
              {exit.direction}: {exit.targetName}
            </button>
          {/each}
        </div>
      {/if}

      <!-- Interactions (Talk to...) -->
      {#if !playState.inDialogue && playState.interactions.length > 0}
        <div class="urd-play__group-label">Interact</div>
        <div class="urd-play__choice-group">
          {#each playState.interactions as inter}
            <button
              class="urd-play__choice-btn"
              class:urd-play__choice-btn--disabled={!inter.available}
              onclick={() => inter.available && playbackService.enterDialogue(inter.dialogueId)}
              disabled={!inter.available}
            >
              Talk to {inter.entityName}
              {#if inter.conditionText}
                <span class="urd-play__cond-tag">{inter.conditionText}</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}

      <!-- Pickups -->
      {#if playState.pickups.length > 0}
        <div class="urd-play__group-label">Items</div>
        <div class="urd-play__choice-group">
          {#each playState.pickups as pickup}
            <button
              class="urd-play__choice-btn"
              onclick={() => playbackService.pickUp(pickup.entityId)}
            >
              Pick up {pickup.entityName}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Inventory bar -->
    <div class="urd-play__inv-bar">
      <span class="urd-play__inv-label">Inventory</span>
      <div class="urd-play__inv-items">
        {#if playState.inventory.length === 0}
          <span class="urd-play__inv-empty">Empty</span>
        {:else}
          {#each playState.inventory as item}
            <span class="urd-play__inv-chip">{item.name}</span>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .urd-play {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: var(--forge-bg-primary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-body, var(--forge-font-family-ui));
    font-size: var(--forge-font-size-sm);
  }

  /* ===== Load screen ===== */

  .urd-play__load {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 16px;
  }

  .urd-play__load-title {
    font-family: var(--forge-font-family-ui);
    font-size: 20px;
    font-weight: 600;
    color: var(--forge-text-primary);
    letter-spacing: 2px;
    text-transform: uppercase;
  }

  .urd-play__load-hint {
    font-family: var(--forge-font-family-mono);
    font-size: 11px;
    color: var(--forge-text-muted);
  }

  .urd-play__load-btn {
    font-family: var(--forge-font-family-ui);
    font-size: 13px;
    padding: 8px 28px;
    background: var(--forge-accent-primary);
    border: 1px solid var(--forge-accent-primary);
    color: var(--forge-bg-primary);
    border-radius: 4px;
    cursor: pointer;
    letter-spacing: 1px;
    text-transform: uppercase;
  }

  .urd-play__load-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .urd-play__load-btn:hover:not(:disabled) {
    filter: brightness(1.15);
  }

  /* ===== Header bar ===== */

  .urd-play__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    padding: 8px 24px;
    background: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .urd-play__header-title {
    font-family: var(--forge-font-family-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--forge-text-primary);
    letter-spacing: 1px;
    text-transform: uppercase;
  }

  .urd-play__header-badge {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-accent-primary);
    background: rgba(106, 170, 90, 0.1);
    border: 1px solid rgba(106, 170, 90, 0.25);
    padding: 2px 8px;
    border-radius: 3px;
  }

  .urd-play__header-actions {
    margin-left: auto;
  }

  .urd-play__hdr-btn {
    font-family: var(--forge-font-family-mono);
    font-size: 11px;
    padding: 4px 12px;
    background: var(--forge-bg-tertiary);
    border: 1px solid var(--forge-border-zone);
    color: var(--forge-text-secondary);
    border-radius: 4px;
    cursor: pointer;
  }

  .urd-play__hdr-btn:hover {
    background: var(--forge-bg-hover);
    color: var(--forge-text-primary);
  }

  /* ===== Viewport ===== */

  .urd-play__viewport {
    flex: 1;
    overflow-y: auto;
    padding: 24px 32px;
    min-height: 0;
    scroll-behavior: smooth;
  }

  .urd-play__viewport::-webkit-scrollbar { width: 6px; }
  .urd-play__viewport::-webkit-scrollbar-track { background: transparent; }
  .urd-play__viewport::-webkit-scrollbar-thumb { background: var(--forge-border-zone); border-radius: 3px; }

  /* ===== Location header ===== */

  .urd-play__loc-header {
    margin-bottom: 20px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--forge-border-zone);
    animation: urd-play-fadeIn 0.5s ease;
  }

  .urd-play__loc-name {
    font-family: var(--forge-font-family-ui);
    font-size: 22px;
    font-weight: 700;
    color: var(--forge-text-primary);
    letter-spacing: 2px;
    margin-bottom: 6px;
  }

  .urd-play__loc-desc {
    font-size: 15px;
    font-style: italic;
    color: var(--forge-text-secondary);
    font-weight: 300;
    line-height: 1.7;
  }

  .urd-play__loc-contents {
    font-family: var(--forge-font-family-mono);
    font-size: 11px;
    color: var(--forge-text-muted);
    margin-top: 10px;
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .urd-play__loc-tag {
    padding: 2px 8px;
    border-radius: 3px;
    border: 1px solid var(--forge-border-zone);
    background: var(--forge-bg-tertiary);
  }

  .urd-play__loc-tag--interactable {
    border-color: rgba(204, 136, 68, 0.3);
    color: var(--forge-runtime-event-dialogue, #cc8844);
  }

  .urd-play__loc-tag--portable {
    border-color: rgba(204, 170, 68, 0.3);
    color: #ccaa44;
  }

  .urd-play__loc-tag--other {
    border-color: rgba(106, 170, 90, 0.3);
    color: var(--forge-accent-primary);
  }

  /* ===== Narrative entries ===== */

  .urd-play__narrative {
    margin-bottom: 16px;
  }

  .urd-play__entry {
    margin-bottom: 12px;
    animation: urd-play-fadeIn 0.4s ease;
  }

  @media (prefers-reduced-motion: reduce) {
    .urd-play__entry, .urd-play__loc-header { animation: none; }
  }

  .urd-play__entry--dlg-prompt {
    background: var(--forge-bg-tertiary);
    border-left: 3px solid rgba(106, 170, 90, 0.4);
    padding: 10px 14px;
    border-radius: 0 6px 6px 0;
  }

  .urd-play__entry--dlg-response {
    background: rgba(106, 170, 90, 0.05);
    border-left: 3px solid var(--forge-accent-primary);
    padding: 10px 14px;
    border-radius: 0 6px 6px 0;
  }

  .urd-play__entry--player-action {
    border-left: 3px solid var(--forge-runtime-event-move, #5588aa);
    padding: 10px 14px;
    background: rgba(85, 136, 170, 0.05);
    border-radius: 0 6px 6px 0;
  }

  .urd-play__entry--narration {
    padding: 4px 0;
    font-style: italic;
    color: var(--forge-text-secondary);
  }

  .urd-play__entry--blocked {
    font-family: var(--forge-font-family-mono);
    font-size: 12px;
    color: var(--forge-graph-node-unreachable, #aa4444);
    border-left: 2px solid var(--forge-graph-node-unreachable, #aa4444);
    padding: 4px 12px;
  }

  .urd-play__entry--system {
    font-family: var(--forge-font-family-mono);
    font-size: 12px;
    color: var(--forge-text-muted);
    padding: 4px 0;
    font-style: italic;
  }

  .urd-play__speaker {
    font-family: var(--forge-font-family-mono);
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    margin-bottom: 3px;
    color: var(--forge-accent-primary);
  }

  .urd-play__entry--player-action .urd-play__speaker {
    color: var(--forge-runtime-event-move, #5588aa);
  }

  .urd-play__text {
    font-size: 15px;
    line-height: 1.7;
    font-weight: 300;
  }

  /* ===== Choices area ===== */

  .urd-play__choices-area {
    flex-shrink: 0;
    padding: 8px 32px 12px;
    background: var(--forge-bg-secondary);
    border-top: 1px solid var(--forge-border-zone);
    max-height: 45%;
    overflow-y: auto;
  }

  .urd-play__group-label {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-muted);
    letter-spacing: 2px;
    text-transform: uppercase;
    margin-bottom: 6px;
    margin-top: 8px;
  }

  .urd-play__group-label:first-child {
    margin-top: 0;
  }

  .urd-play__choice-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 8px;
  }

  .urd-play__choice-btn {
    font-family: var(--forge-font-family-body, var(--forge-font-family-ui));
    font-size: 14px;
    padding: 8px 16px;
    background: var(--forge-bg-tertiary);
    border: 1px solid var(--forge-border-zone);
    color: var(--forge-text-primary);
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .urd-play__choice-btn:hover:not(:disabled) {
    background: var(--forge-bg-hover);
    border-color: rgba(106, 170, 90, 0.4);
    color: var(--forge-text-primary);
    padding-left: 22px;
  }

  .urd-play__choice-btn--disabled {
    opacity: 0.35;
    cursor: not-allowed;
    border-style: dashed;
  }

  .urd-play__choice-btn--disabled:hover {
    padding-left: 16px;
    background: var(--forge-bg-tertiary);
    border-color: var(--forge-border-zone);
  }

  .urd-play__cond-tag {
    font-family: var(--forge-font-family-mono);
    font-size: 9px;
    padding: 1px 6px;
    border-radius: 2px;
    margin-left: auto;
    flex-shrink: 0;
    color: var(--forge-graph-node-unreachable, #aa4444);
    background: rgba(170, 68, 68, 0.1);
  }

  .urd-play__cond-tag--ok {
    color: var(--forge-accent-primary);
    background: rgba(106, 170, 90, 0.1);
  }

  .urd-play__exit-btn {
    font-family: var(--forge-font-family-body, var(--forge-font-family-ui));
    font-size: 13px;
    padding: 6px 14px;
    background: transparent;
    border: 1px solid var(--forge-border-zone);
    color: var(--forge-text-secondary);
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s;
  }

  .urd-play__exit-btn:hover {
    background: var(--forge-bg-tertiary);
    border-color: rgba(106, 170, 90, 0.4);
    color: var(--forge-text-primary);
  }

  .urd-play__exit-btn--blocked {
    opacity: 0.4;
    cursor: pointer;
    border-style: dashed;
  }

  .urd-play__exit-btn--blocked:hover {
    border-color: var(--forge-graph-node-unreachable, #aa4444);
    color: var(--forge-graph-node-unreachable, #aa4444);
    opacity: 0.7;
  }

  /* ===== Inventory bar ===== */

  .urd-play__inv-bar {
    flex-shrink: 0;
    background: var(--forge-bg-tertiary);
    border-top: 1px solid var(--forge-border-zone);
    padding: 6px 32px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .urd-play__inv-label {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    color: var(--forge-text-muted);
    letter-spacing: 1.5px;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .urd-play__inv-items {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .urd-play__inv-chip {
    font-family: var(--forge-font-family-mono);
    font-size: 11px;
    padding: 2px 10px;
    background: rgba(204, 170, 68, 0.08);
    border: 1px solid rgba(204, 170, 68, 0.25);
    color: #ccaa44;
    border-radius: 3px;
  }

  .urd-play__inv-empty {
    font-family: var(--forge-font-family-mono);
    font-size: 11px;
    color: var(--forge-text-muted);
    font-style: italic;
  }

  /* ===== Game over ===== */

  .urd-play__game-over {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px;
    background: rgba(0, 0, 0, 0.3);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .urd-play__game-over-title {
    font-family: var(--forge-font-family-ui);
    font-size: 24px;
    font-weight: 700;
    color: var(--forge-text-primary);
    letter-spacing: 3px;
    text-transform: uppercase;
    margin-bottom: 16px;
  }

  /* ===== Phase badge ===== */

  .urd-play__header-badge--phase {
    color: var(--forge-runtime-event-dialogue, #cc8844);
    background: rgba(204, 136, 68, 0.1);
    border: 1px solid rgba(204, 136, 68, 0.25);
  }

  /* ===== Action target ===== */

  .urd-play__action-target {
    font-family: var(--forge-font-family-mono);
    font-size: 11px;
    color: var(--forge-text-muted);
  }

  /* ===== Animations ===== */

  @keyframes urd-play-fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
