<script lang="ts">
  /**
   * InspectorSection — collapsible section within an inspector panel.
   */

  import { untrack } from 'svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    collapsed?: boolean;
    children: Snippet;
  }

  let { title, collapsed = false, children }: Props = $props();

  let isCollapsed: boolean = $state(untrack(() => collapsed));

  function toggle(): void {
    isCollapsed = !isCollapsed;
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      toggle();
    }
  }
</script>

<div class="forge-inspector-section">
  <div
    class="forge-inspector-section__header"
    role="button"
    tabindex="0"
    aria-expanded={!isCollapsed}
    onclick={toggle}
    onkeydown={handleKeydown}
  >
    <span class="forge-inspector-section__toggle">{isCollapsed ? '\u25B8' : '\u25BE'}</span>
    <span class="forge-inspector-section__title">{title}</span>
  </div>
  {#if !isCollapsed}
    <div class="forge-inspector-section__body">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .forge-inspector-section {
    margin-bottom: var(--forge-space-sm);
  }

  .forge-inspector-section__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-xs) 0;
    cursor: pointer;
    user-select: none;
  }

  .forge-inspector-section__header:hover {
    color: var(--forge-text-primary);
  }

  .forge-inspector-section__toggle {
    flex-shrink: 0;
    width: 12px;
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
  }

  .forge-inspector-section__title {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    font-weight: 600;
    color: var(--forge-text-secondary);
  }

  .forge-inspector-section__body {
    padding-left: var(--forge-space-lg);
  }
</style>
