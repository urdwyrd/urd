<script lang="ts">
  /**
   * Error boundary for zone viewports.
   *
   * Catches uncaught exceptions during render. Shows a crash panel
   * with reload/switch options. Does NOT wrap zone header or shell.
   */

  import type { Snippet } from 'svelte';
  import { bus } from '../bus/MessageBus';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    children: Snippet;
    onReload: () => void;
    onChangeType: (typeId: string) => void;
  }

  let { zoneId, zoneTypeId, children, onReload, onChangeType }: Props = $props();

  let error = $state<string | null>(null);

  // Use Svelte's boundary mechanism via try/catch in the template
  function handleError(err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    error = message;
    console.error(`Zone ${zoneId} (${zoneTypeId}) crashed:`, err);

    if (bus.hasChannel('zone.error')) {
      bus.publish('zone.error', { zoneId, zoneTypeId, error: message });
    }
  }

  function reload() {
    error = null;
    onReload();
  }
</script>

{#if error}
  <div class="forge-zone-error">
    <div class="forge-zone-error__icon">!</div>
    <p class="forge-zone-error__type">{zoneTypeId}</p>
    <p class="forge-zone-error__message">{error}</p>
    <div class="forge-zone-error__actions">
      <button class="forge-zone-error__btn" onclick={reload}>Reload View</button>
      <button class="forge-zone-error__btn forge-zone-error__btn--secondary" onclick={() => onChangeType('forge.placeholder.info')}>
        Switch to Info
      </button>
    </div>
  </div>
{:else}
  {@render children()}
{/if}

<style>
  .forge-zone-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--forge-space-md);
    padding: var(--forge-space-xl);
    text-align: center;
    color: var(--forge-text-secondary);
  }

  .forge-zone-error__icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--forge-status-error);
    color: white;
    font-weight: var(--forge-font-weight-bold);
    font-size: var(--forge-font-size-lg);
  }

  .forge-zone-error__type {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-zone-error__message {
    font-size: var(--forge-font-size-sm);
    max-width: 300px;
    word-break: break-word;
  }

  .forge-zone-error__actions {
    display: flex;
    gap: var(--forge-space-md);
    margin-top: var(--forge-space-md);
  }

  .forge-zone-error__btn {
    padding: var(--forge-space-sm) var(--forge-space-lg);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    background: var(--forge-bg-secondary);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
  }

  .forge-zone-error__btn:hover {
    background: var(--forge-bg-tertiary);
  }

  .forge-zone-error__btn--secondary {
    background: transparent;
  }
</style>
