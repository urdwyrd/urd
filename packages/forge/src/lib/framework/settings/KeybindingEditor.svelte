<script lang="ts">
  /**
   * KeybindingEditor — filterable table of all commands with editable keybindings.
   *
   * Click a keybinding cell to enter capture mode. Press a key combination to set it.
   * Escape cancels. Conflict detection warns before overwriting.
   */

  import { commandRegistry, type CommandDefinition, formatKeybinding, normaliseKeybinding } from '../commands/CommandRegistry';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: unknown;
    onStateChange: (state: unknown) => void;
  }

  let { }: Props = $props();

  let searchQuery = $state('');
  let captureCommandId = $state<string | null>(null);
  let capturedKey = $state<string | null>(null);

  const commands = $derived.by(() => {
    const all = commandRegistry.list();
    if (!searchQuery.trim()) return all;
    const q = searchQuery.toLowerCase();
    return all.filter((cmd) =>
      cmd.title.toLowerCase().includes(q) ||
      cmd.category.toLowerCase().includes(q) ||
      cmd.id.toLowerCase().includes(q) ||
      (cmd.keybinding ?? '').toLowerCase().includes(q)
    );
  });

  function startCapture(commandId: string): void {
    captureCommandId = commandId;
    capturedKey = null;
  }

  function cancelCapture(): void {
    captureCommandId = null;
    capturedKey = null;
  }

  function handleCaptureKeydown(e: KeyboardEvent): void {
    if (!captureCommandId) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.key === 'Escape') {
      cancelCapture();
      return;
    }

    // Ignore bare modifier keys
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;

    const binding = formatKeybinding(e);
    const normalised = normaliseKeybinding(binding);

    // Check for conflicts
    const existing = commandRegistry.resolveByKeybinding(normalised);
    if (existing && existing.id !== captureCommandId) {
      // Remove conflicting binding
      commandRegistry.rebind(existing.id, null);
    }

    commandRegistry.rebind(captureCommandId, binding);
    captureCommandId = null;
    capturedKey = null;
  }

  function clearBinding(commandId: string): void {
    commandRegistry.rebind(commandId, null);
  }

  function formatDisplayKeybinding(kb: string | undefined): string {
    if (!kb) return '—';
    return kb
      .split('+')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' + ');
  }
</script>

<div class="forge-keybindings">
  <div class="forge-keybindings__header">
    <input
      bind:value={searchQuery}
      class="forge-keybindings__search"
      type="text"
      placeholder="Search commands…"
      spellcheck="false"
    />
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="forge-keybindings__table"
    onkeydown={captureCommandId ? handleCaptureKeydown : undefined}
  >
    <div class="forge-keybindings__row forge-keybindings__row--header">
      <span class="forge-keybindings__cell forge-keybindings__cell--command">Command</span>
      <span class="forge-keybindings__cell forge-keybindings__cell--category">Category</span>
      <span class="forge-keybindings__cell forge-keybindings__cell--binding">Keybinding</span>
      <span class="forge-keybindings__cell forge-keybindings__cell--actions"></span>
    </div>

    {#each commands as cmd}
      <div class="forge-keybindings__row" class:forge-keybindings__row--capturing={captureCommandId === cmd.id}>
        <span class="forge-keybindings__cell forge-keybindings__cell--command forge-selectable">{cmd.title}</span>
        <span class="forge-keybindings__cell forge-keybindings__cell--category">{cmd.category}</span>
        <button
          class="forge-keybindings__cell forge-keybindings__cell--binding forge-keybindings__binding-btn"
          onclick={() => startCapture(cmd.id)}
        >
          {#if captureCommandId === cmd.id}
            <span class="forge-keybindings__capture">Press keys…</span>
          {:else}
            {formatDisplayKeybinding(cmd.keybinding)}
          {/if}
        </button>
        <span class="forge-keybindings__cell forge-keybindings__cell--actions">
          {#if cmd.keybinding}
            <button class="forge-keybindings__clear" onclick={() => clearBinding(cmd.id)} title="Clear keybinding">
              ✕
            </button>
          {/if}
        </span>
      </div>
    {/each}

    {#if commands.length === 0}
      <div class="forge-keybindings__empty">No commands match your search</div>
    {/if}
  </div>
</div>

<style>
  .forge-keybindings {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .forge-keybindings__header {
    padding: var(--forge-space-sm) var(--forge-space-md);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-keybindings__search {
    width: 100%;
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    font-family: inherit;
    outline: none;
  }

  .forge-keybindings__search::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-keybindings__table {
    flex: 1;
    overflow-y: auto;
    font-size: var(--forge-font-size-sm);
  }

  .forge-keybindings__row {
    display: flex;
    align-items: center;
    padding: var(--forge-space-xs) var(--forge-space-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .forge-keybindings__row:hover {
    background: var(--forge-table-row-hover);
  }

  .forge-keybindings__row--header {
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    position: sticky;
    top: 0;
    background: var(--forge-bg-secondary);
    z-index: 1;
  }

  .forge-keybindings__row--header:hover {
    background: var(--forge-bg-secondary);
  }

  .forge-keybindings__row--capturing {
    background: rgba(100, 200, 255, 0.08);
  }

  .forge-keybindings__cell {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-keybindings__cell--command {
    flex: 2;
    color: var(--forge-text-primary);
  }

  .forge-keybindings__cell--category {
    flex: 1;
    color: var(--forge-text-muted);
  }

  .forge-keybindings__cell--binding {
    flex: 1;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-keybindings__cell--actions {
    width: 30px;
    flex-shrink: 0;
    text-align: center;
  }

  .forge-keybindings__binding-btn {
    border: none;
    background: transparent;
    color: var(--forge-text-secondary);
    cursor: pointer;
    padding: 2px var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    text-align: left;
  }

  .forge-keybindings__binding-btn:hover {
    background: var(--forge-bg-tertiary);
  }

  .forge-keybindings__capture {
    color: var(--forge-status-info);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .forge-keybindings__clear {
    border: none;
    background: transparent;
    color: var(--forge-text-muted);
    cursor: pointer;
    font-size: var(--forge-font-size-xs);
    padding: 2px;
    border-radius: var(--forge-radius-sm);
  }

  .forge-keybindings__clear:hover {
    color: var(--forge-status-error, #c66);
  }

  .forge-keybindings__empty {
    padding: var(--forge-space-xl);
    text-align: center;
    color: var(--forge-text-muted);
    font-style: italic;
  }
</style>
