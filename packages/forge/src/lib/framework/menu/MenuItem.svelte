<script lang="ts">
  /**
   * Single menu item — maps to a command.
   */

  import { commandRegistry } from '../commands/CommandRegistry';

  interface Props {
    commandId: string;
    label?: string;
    disabled?: boolean;
    onClose: () => void;
  }

  let { commandId, label, disabled = false, onClose }: Props = $props();

  let command = $derived(commandRegistry.get(commandId));
  let displayLabel = $derived(label ?? command?.title ?? commandId);
  let keybinding = $derived(command?.keybinding);

  function handleClick() {
    if (disabled) return;
    commandRegistry.execute(commandId);
    onClose();
  }
</script>

<button
  class="forge-menu-item"
  class:forge-menu-item--disabled={disabled}
  role="menuitem"
  {disabled}
  onclick={handleClick}
>
  <span class="forge-menu-item__label">{displayLabel}</span>
  {#if keybinding}
    <span class="forge-menu-item__shortcut">{keybinding}</span>
  {/if}
</button>

<style>
  .forge-menu-item {
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

  .forge-menu-item:hover:not(:disabled) {
    background: var(--forge-bg-tertiary);
  }

  .forge-menu-item--disabled {
    color: var(--forge-text-muted);
    cursor: default;
  }

  .forge-menu-item__shortcut {
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }
</style>
