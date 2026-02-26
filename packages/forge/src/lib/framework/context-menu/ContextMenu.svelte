<script lang="ts">
  /**
   * ContextMenu — rendered menu positioned at cursor.
   */

  import type { ContextMenuItem } from '../types';
  import { commandRegistry } from '../commands/CommandRegistry';
  import { onMount } from 'svelte';

  interface Props {
    items: ContextMenuItem[];
    x: number;
    y: number;
    onClose: () => void;
  }

  let { items, x, y, onClose }: Props = $props();

  let menuEl: HTMLDivElement | undefined = $state();

  onMount(() => {
    // Close on click outside
    const handler = (e: MouseEvent) => {
      if (menuEl && !menuEl.contains(e.target as Node)) {
        onClose();
      }
    };
    // Close on Escape
    const keyHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };

    // Delay to avoid immediate close from the triggering click
    requestAnimationFrame(() => {
      document.addEventListener('mousedown', handler);
      document.addEventListener('keydown', keyHandler);
    });

    return () => {
      document.removeEventListener('mousedown', handler);
      document.removeEventListener('keydown', keyHandler);
    };
  });

  function handleClick(item: ContextMenuItem) {
    if (item.disabled || item.separator) return;
    commandRegistry.execute(item.commandId, item.commandArgs);
    onClose();
  }

  // Ensure menu stays within viewport
  let adjustedX = $derived(Math.min(x, window.innerWidth - 200));
  let adjustedY = $derived(Math.min(y, window.innerHeight - 300));
</script>

<div
  class="forge-context-menu"
  style:left="{adjustedX}px"
  style:top="{adjustedY}px"
  bind:this={menuEl}
  role="menu"
>
  {#each items as item}
    {#if item.separator}
      <div class="forge-context-menu__separator" role="separator"></div>
    {:else}
      <button
        class="forge-context-menu__item"
        class:forge-context-menu__item--disabled={item.disabled}
        role="menuitem"
        disabled={item.disabled}
        onclick={() => handleClick(item)}
      >
        <span class="forge-context-menu__label">{item.label}</span>
        {#if item.keybinding}
          <span class="forge-context-menu__shortcut">{item.keybinding}</span>
        {/if}
      </button>
    {/if}
  {/each}
</div>

<style>
  .forge-context-menu {
    position: fixed;
    min-width: 160px;
    max-width: 280px;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    z-index: var(--forge-z-dropdown);
    padding: var(--forge-space-xs) 0;
    overflow: hidden;
  }

  .forge-context-menu__separator {
    height: 1px;
    background: var(--forge-border-zone);
    margin: var(--forge-space-xs) 0;
  }

  .forge-context-menu__item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-lg);
    border: none;
    background: transparent;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    text-align: left;
    cursor: pointer;
    gap: var(--forge-space-xl);
  }

  .forge-context-menu__item:hover:not(:disabled) {
    background: var(--forge-bg-tertiary);
  }

  .forge-context-menu__item--disabled {
    color: var(--forge-text-muted);
    cursor: default;
  }

  .forge-context-menu__shortcut {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }
</style>
