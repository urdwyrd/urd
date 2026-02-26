<script lang="ts">
  /**
   * Command palette — modal overlay with fuzzy search over commands and views.
   *
   * Opens when FocusMode is 'commandPalette'. Escape calls popMode().
   * Keyboard navigation: ArrowUp/Down to navigate, Enter to execute.
   */

  import { commandRegistry, type CommandDefinition } from './CommandRegistry';
  import { viewRegistry, type ViewRegistration } from '../views/ViewRegistry';
  import { focusService } from '../focus/FocusService.svelte';
  import { onMount } from 'svelte';

  interface PaletteItem {
    kind: 'command' | 'view';
    id: string;
    title: string;
    category: string;
    keybinding?: string;
  }

  let query = $state('');
  let selectedIndex = $state(0);
  let inputEl: HTMLInputElement | undefined = $state(undefined);

  function buildItems(): PaletteItem[] {
    const commands: PaletteItem[] = commandRegistry.list().map((cmd: CommandDefinition) => ({
      kind: 'command',
      id: cmd.id,
      title: cmd.title,
      category: cmd.category,
      keybinding: cmd.keybinding,
    }));

    const views: PaletteItem[] = viewRegistry.list().map((v: ViewRegistration) => ({
      kind: 'view',
      id: v.id,
      title: v.name,
      category: v.category,
    }));

    return [...commands, ...views];
  }

  const allItems = buildItems();

  const filtered = $derived.by(() => {
    if (!query.trim()) return allItems;
    const q = query.toLowerCase();
    return allItems.filter((item) => {
      const haystack = `${item.title} ${item.category} ${item.id}`.toLowerCase();
      return fuzzyMatch(q, haystack);
    });
  });

  // Reset selected index when filter changes
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    filtered.length;
    selectedIndex = 0;
  });

  function fuzzyMatch(needle: string, haystack: string): boolean {
    let ni = 0;
    for (let hi = 0; hi < haystack.length && ni < needle.length; hi++) {
      if (haystack[hi] === needle[ni]) {
        ni++;
      }
    }
    return ni === needle.length;
  }

  function close(): void {
    focusService.popMode();
  }

  function executeItem(item: PaletteItem): void {
    close();
    if (item.kind === 'command') {
      commandRegistry.execute(item.id);
    }
    // View navigation will be handled via NavigationBroker in later commits
  }

  function handleKeydown(e: KeyboardEvent): void {
    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        close();
        break;
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
        scrollSelectedIntoView();
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        scrollSelectedIntoView();
        break;
      case 'Enter':
        e.preventDefault();
        if (filtered[selectedIndex]) {
          executeItem(filtered[selectedIndex]);
        }
        break;
    }
  }

  function scrollSelectedIntoView(): void {
    const el = document.querySelector('.forge-palette__item--selected');
    el?.scrollIntoView({ block: 'nearest' });
  }

  function formatKeybinding(kb: string): string {
    return kb
      .split('+')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join('+');
  }

  onMount(() => {
    inputEl?.focus();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="forge-palette__backdrop" onmousedown={close}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="forge-palette" onmousedown={(e) => e.stopPropagation()}>
    <input
      bind:this={inputEl}
      bind:value={query}
      onkeydown={handleKeydown}
      class="forge-palette__input"
      type="text"
      placeholder="Type a command or view name…"
      spellcheck="false"
      autocomplete="off"
    />
    <div class="forge-palette__list" role="listbox">
      {#each filtered as item, i}
        <button
          class="forge-palette__item"
          class:forge-palette__item--selected={i === selectedIndex}
          role="option"
          aria-selected={i === selectedIndex}
          onmouseenter={() => { selectedIndex = i; }}
          onclick={() => executeItem(item)}
        >
          <span class="forge-palette__item-kind">{item.kind === 'command' ? '▸' : '◆'}</span>
          <span class="forge-palette__item-title">{item.title}</span>
          <span class="forge-palette__item-category">{item.category}</span>
          {#if item.keybinding}
            <span class="forge-palette__item-keybinding">{formatKeybinding(item.keybinding)}</span>
          {/if}
        </button>
      {/each}
      {#if filtered.length === 0}
        <div class="forge-palette__empty">No matches</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .forge-palette__backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    justify-content: center;
    padding-top: 15vh;
    background: rgba(0, 0, 0, 0.4);
  }

  .forge-palette {
    width: min(600px, 90vw);
    max-height: 60vh;
    display: flex;
    flex-direction: column;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    align-self: flex-start;
  }

  .forge-palette__input {
    padding: var(--forge-space-md) var(--forge-space-lg);
    border: none;
    border-bottom: 1px solid var(--forge-border-zone);
    background: transparent;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-md);
    font-family: inherit;
    outline: none;
  }

  .forge-palette__input::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-palette__list {
    flex: 1;
    overflow-y: auto;
    padding: var(--forge-space-xs) 0;
  }

  .forge-palette__item {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-lg);
    border: none;
    background: transparent;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    font-family: inherit;
    cursor: pointer;
    text-align: left;
  }

  .forge-palette__item--selected {
    background: var(--forge-accent-primary);
    color: var(--forge-text-primary);
  }

  .forge-palette__item-kind {
    flex-shrink: 0;
    width: 1.2em;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
  }

  .forge-palette__item--selected .forge-palette__item-kind {
    color: inherit;
  }

  .forge-palette__item-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-palette__item-category {
    flex-shrink: 0;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-xs);
  }

  .forge-palette__item--selected .forge-palette__item-category {
    color: inherit;
    opacity: 0.7;
  }

  .forge-palette__item-keybinding {
    flex-shrink: 0;
    padding: 1px var(--forge-space-xs);
    border-radius: 3px;
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-secondary);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-palette__item--selected .forge-palette__item-keybinding {
    background: rgba(255, 255, 255, 0.15);
    color: inherit;
  }

  .forge-palette__empty {
    padding: var(--forge-space-xl);
    text-align: center;
    color: var(--forge-text-muted);
    font-style: italic;
  }
</style>
