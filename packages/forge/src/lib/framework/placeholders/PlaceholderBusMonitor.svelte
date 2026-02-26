<script lang="ts">
  /**
   * PlaceholderBusMonitor — live log of all message bus events.
   */

  import { bus } from '../bus/MessageBus';
  import { onMount } from 'svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: unknown;
    onStateChange: (state: unknown) => void;
  }

  let { }: Props = $props();

  interface BusEvent {
    channelId: string;
    payload: string;
    timestamp: number;
  }

  let events = $state<BusEvent[]>([]);

  onMount(() => {
    const unsubs: (() => void)[] = [];

    // Subscribe to all registered channels
    for (const channelId of bus.listChannels()) {
      unsubs.push(
        bus.subscribe(channelId, (payload) => {
          events = [
            {
              channelId,
              payload: JSON.stringify(payload, null, 0)?.slice(0, 200) ?? 'undefined',
              timestamp: Date.now(),
            },
            ...events.slice(0, 99), // keep last 100
          ];
        })
      );
    }

    return () => unsubs.forEach((u) => u());
  });

  function formatTime(ts: number): string {
    const d = new Date(ts);
    return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}:${d.getSeconds().toString().padStart(2, '0')}.${d.getMilliseconds().toString().padStart(3, '0')}`;
  }

  function isCompilerEvent(channelId: string): boolean {
    return channelId.startsWith('compiler.');
  }

  function formatCompilerPayload(channelId: string, payload: string): string {
    try {
      const data = JSON.parse(payload);
      if (channelId === 'compiler.completed' && data.durationMs !== undefined) {
        const entities = data.worldCounts?.entities ?? '?';
        const locations = data.worldCounts?.locations ?? '?';
        return `${data.durationMs}ms — ${entities} entities, ${locations} locations`;
      }
      if (channelId === 'compiler.started' && data.inputFileCount !== undefined) {
        return `${data.inputFileCount} file(s)`;
      }
      if (channelId === 'compiler.error') {
        return data.error ?? payload;
      }
    } catch {
      // Fall through to raw payload
    }
    return payload;
  }
</script>

<div class="forge-bus-monitor">
  <div class="forge-bus-monitor__header">Bus Monitor — {events.length} events</div>
  <div class="forge-bus-monitor__list">
    {#each events as event}
      <div class="forge-bus-monitor__event" class:forge-bus-monitor__event--compiler={isCompilerEvent(event.channelId)}>
        <span class="forge-bus-monitor__time">{formatTime(event.timestamp)}</span>
        <span class="forge-bus-monitor__channel forge-selectable">{event.channelId}</span>
        <span class="forge-bus-monitor__payload forge-selectable">
          {isCompilerEvent(event.channelId) ? formatCompilerPayload(event.channelId, event.payload) : event.payload}
        </span>
      </div>
    {/each}
    {#if events.length === 0}
      <div class="forge-bus-monitor__empty">No events yet</div>
    {/if}
  </div>
</div>

<style>
  .forge-bus-monitor {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .forge-bus-monitor__header {
    padding: var(--forge-space-sm) var(--forge-space-md);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-bus-monitor__list {
    flex: 1;
    overflow-y: auto;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-bus-monitor__event {
    display: flex;
    gap: var(--forge-space-md);
    padding: var(--forge-space-xs) var(--forge-space-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .forge-bus-monitor__event:hover {
    background: var(--forge-table-row-hover);
  }

  .forge-bus-monitor__time {
    color: var(--forge-text-muted);
    flex-shrink: 0;
  }

  .forge-bus-monitor__channel {
    color: var(--forge-status-info);
    flex-shrink: 0;
    min-width: 120px;
  }

  .forge-bus-monitor__payload {
    color: var(--forge-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-bus-monitor__event--compiler {
    background: rgba(100, 200, 100, 0.05);
  }

  .forge-bus-monitor__event--compiler .forge-bus-monitor__channel {
    color: var(--forge-status-success, #6b8);
  }

  .forge-bus-monitor__empty {
    padding: var(--forge-space-xl);
    text-align: center;
    color: var(--forge-text-muted);
    font-style: italic;
  }
</style>
