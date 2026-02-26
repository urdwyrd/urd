<script lang="ts">
  /**
   * PlaceholderInfo — displays zone metadata: ID, type, dimensions.
   * Updates reactively on resize.
   */

  import { onMount } from 'svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: unknown;
    onStateChange: (state: unknown) => void;
  }

  let { zoneId, zoneTypeId }: Props = $props();

  let width = $state(0);
  let height = $state(0);
  let containerEl: HTMLDivElement | undefined = $state();

  onMount(() => {
    if (!containerEl) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        width = Math.round(entry.contentRect.width);
        height = Math.round(entry.contentRect.height);
      }
    });

    observer.observe(containerEl);
    return () => observer.disconnect();
  });
</script>

<div class="forge-placeholder-info" bind:this={containerEl}>
  <table class="forge-placeholder-info__table">
    <tbody>
      <tr>
        <td class="forge-placeholder-info__label">Zone ID</td>
        <td class="forge-placeholder-info__value forge-selectable">{zoneId}</td>
      </tr>
      <tr>
        <td class="forge-placeholder-info__label">Type</td>
        <td class="forge-placeholder-info__value forge-selectable">{zoneTypeId}</td>
      </tr>
      <tr>
        <td class="forge-placeholder-info__label">Width</td>
        <td class="forge-placeholder-info__value">{width}px</td>
      </tr>
      <tr>
        <td class="forge-placeholder-info__label">Height</td>
        <td class="forge-placeholder-info__value">{height}px</td>
      </tr>
    </tbody>
  </table>
</div>

<style>
  .forge-placeholder-info {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: var(--forge-space-xl);
  }

  .forge-placeholder-info__table {
    border-collapse: collapse;
  }

  .forge-placeholder-info__label {
    padding: var(--forge-space-xs) var(--forge-space-lg) var(--forge-space-xs) 0;
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    text-align: right;
    white-space: nowrap;
  }

  .forge-placeholder-info__value {
    padding: var(--forge-space-xs) 0;
    font-size: var(--forge-font-size-sm);
    font-family: var(--forge-font-family-mono);
    color: var(--forge-text-primary);
  }
</style>
