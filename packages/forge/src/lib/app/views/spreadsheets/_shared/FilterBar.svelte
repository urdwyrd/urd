<script lang="ts">
  /**
   * FilterBar — text input for local row filtering in spreadsheet views.
   */

  interface Props {
    filterText: string;
    placeholder?: string;
    onFilterChange: (text: string) => void;
  }

  let { filterText, placeholder = 'Filter\u2026', onFilterChange }: Props = $props();

  function handleInput(e: Event): void {
    const value = (e.target as HTMLInputElement).value;
    onFilterChange(value);
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      onFilterChange('');
    }
  }
</script>

<div class="forge-filter-bar">
  <input
    class="forge-filter-bar__input"
    type="text"
    value={filterText}
    {placeholder}
    oninput={handleInput}
    onkeydown={handleKeydown}
    spellcheck="false"
    autocomplete="off"
  />
  {#if filterText}
    <button
      class="forge-filter-bar__clear"
      onclick={() => onFilterChange('')}
      aria-label="Clear filter"
    >&times;</button>
  {/if}
</div>

<style>
  .forge-filter-bar {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-filter-bar__input {
    flex: 1;
    height: 22px;
    padding: 0 var(--forge-space-sm);
    background-color: var(--forge-bg-tertiary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    outline: none;
  }

  .forge-filter-bar__input:focus {
    border-color: var(--forge-accent-primary);
  }

  .forge-filter-bar__input::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-filter-bar__clear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    margin-left: var(--forge-space-xs);
    padding: 0;
    background: none;
    border: none;
    color: var(--forge-text-muted);
    font-size: 14px;
    cursor: pointer;
    border-radius: var(--forge-radius-sm);
  }

  .forge-filter-bar__clear:hover {
    color: var(--forge-text-primary);
    background-color: var(--forge-bg-tertiary);
  }
</style>
