<script lang="ts">
  /**
   * PlaceholderCommandLog — live log of command executions.
   */

  import { subscribeToBus } from '../bus/useBusChannel';
  import { onMount } from 'svelte';
  import type { CommandExecution } from '../commands/CommandRegistry';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: unknown;
    onStateChange: (state: unknown) => void;
  }

  let { }: Props = $props();

  let executions = $state<CommandExecution[]>([]);

  onMount(() => {
    const unsub = subscribeToBus('command.executed', (payload) => {
      const execution = payload as CommandExecution;
      executions = [execution, ...executions.slice(0, 99)];
    });
    return unsub;
  });

  function formatTime(ts: number): string {
    const d = new Date(ts);
    return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}:${d.getSeconds().toString().padStart(2, '0')}`;
  }
</script>

<div class="forge-command-log">
  <div class="forge-command-log__header">Command Log — {executions.length} executions</div>
  <div class="forge-command-log__list">
    {#each executions as exec}
      <div class="forge-command-log__entry">
        <span class="forge-command-log__time">{formatTime(exec.timestamp)}</span>
        <span class="forge-command-log__cmd forge-selectable">{exec.commandId}</span>
        {#if exec.args.length > 0}
          <span class="forge-command-log__args forge-selectable">{JSON.stringify(exec.args)}</span>
        {/if}
      </div>
    {/each}
    {#if executions.length === 0}
      <div class="forge-command-log__empty">No commands executed yet</div>
    {/if}
  </div>
</div>

<style>
  .forge-command-log {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .forge-command-log__header {
    padding: var(--forge-space-sm) var(--forge-space-md);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-command-log__list {
    flex: 1;
    overflow-y: auto;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-command-log__entry {
    display: flex;
    gap: var(--forge-space-md);
    padding: var(--forge-space-xs) var(--forge-space-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .forge-command-log__entry:hover {
    background: var(--forge-table-row-hover);
  }

  .forge-command-log__time {
    color: var(--forge-text-muted);
    flex-shrink: 0;
  }

  .forge-command-log__cmd {
    color: var(--forge-accent-primary);
    flex-shrink: 0;
  }

  .forge-command-log__args {
    color: var(--forge-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-command-log__empty {
    padding: var(--forge-space-xl);
    text-align: center;
    color: var(--forge-text-muted);
    font-style: italic;
  }
</style>
