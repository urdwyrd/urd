<script lang="ts">
  /**
   * InspectorPanel — shared wrapper with title and empty state.
   */

  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    emptyMessage?: string;
    hasContent: boolean;
    children: Snippet;
  }

  let { title, emptyMessage = 'Nothing selected', hasContent, children }: Props = $props();
</script>

<div class="forge-inspector-panel">
  <div class="forge-inspector-panel__header">
    <span class="forge-inspector-panel__title">{title}</span>
  </div>
  <div class="forge-inspector-panel__body">
    {#if hasContent}
      {@render children()}
    {:else}
      <div class="forge-inspector-panel__empty">
        {emptyMessage}
      </div>
    {/if}
  </div>
</div>

<style>
  .forge-inspector-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-inspector-panel__header {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-inspector-panel__title {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-inspector-panel__body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-md);
  }

  .forge-inspector-panel__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }
</style>
