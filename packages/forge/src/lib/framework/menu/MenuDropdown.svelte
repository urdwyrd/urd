<script lang="ts">
  /**
   * MenuDropdown — individual dropdown menu in the global menu bar.
   */

  import MenuItem from './MenuItem.svelte';
  import MenuSeparator from './MenuSeparator.svelte';
  import { getGroupedMenuContributions, type MenuId } from './MenuRegistry';
  import { onMount } from 'svelte';

  interface Props {
    menuId: MenuId;
    label: string;
    open: boolean;
    onToggle: () => void;
    onClose: () => void;
    onHover: () => void;
  }

  let { menuId, label, open, onToggle, onClose, onHover }: Props = $props();

  let groups = $derived(getGroupedMenuContributions(menuId));
  let dropdownEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (dropdownEl && !dropdownEl.contains(e.target as Node)) {
        // Check if click is on our own trigger button
        const btn = dropdownEl.previousElementSibling;
        if (btn && btn.contains(e.target as Node)) return;
        onClose();
      }
    };
    const keyHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('mousedown', handler);
    document.addEventListener('keydown', keyHandler);
    return () => {
      document.removeEventListener('mousedown', handler);
      document.removeEventListener('keydown', keyHandler);
    };
  });
</script>

<div class="forge-menu-dropdown" role="menubar">
  <button
    class="forge-menu-dropdown__trigger"
    class:forge-menu-dropdown__trigger--open={open}
    onclick={onToggle}
    onmouseenter={onHover}
    role="menuitem"
    aria-haspopup="true"
    aria-expanded={open}
  >
    {label}
  </button>

  {#if open}
    <div class="forge-menu-dropdown__panel" bind:this={dropdownEl} role="menu">
      {#each groups as group, gi}
        {#if gi > 0}
          <MenuSeparator />
        {/if}
        {#each group as item}
          <MenuItem
            commandId={item.commandId}
            label={item.label}
            {onClose}
          />
        {/each}
      {/each}
      {#if groups.length === 0}
        <div class="forge-menu-dropdown__empty">No items</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-menu-dropdown {
    position: relative;
  }

  .forge-menu-dropdown__trigger {
    padding: var(--forge-space-xs) var(--forge-space-md);
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-secondary);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
  }

  .forge-menu-dropdown__trigger:hover,
  .forge-menu-dropdown__trigger--open {
    background: var(--forge-bg-secondary);
    color: var(--forge-text-primary);
  }

  .forge-menu-dropdown__panel {
    position: absolute;
    top: 100%;
    left: 0;
    min-width: 200px;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    z-index: var(--forge-z-dropdown);
    padding: var(--forge-space-xs) 0;
  }

  .forge-menu-dropdown__empty {
    padding: var(--forge-space-sm) var(--forge-space-lg);
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-sm);
    font-style: italic;
  }
</style>
