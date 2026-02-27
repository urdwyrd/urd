<script lang="ts">
  /**
   * RegexSearch — pattern-based search across source files via bufferMap.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { bufferMap } from '$lib/app/compiler/BufferMap';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';

  interface Props { zoneId: string; zoneTypeId: string; state: null; onStateChange: (newState: unknown) => void; }
  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  interface SearchResult { file: string; line: number; content: string; }

  let query: string = $state('');
  let results: SearchResult[] = $state([]);
  let selectedIndex: number = $state(0);
  let error: string | null = $state(null);
  let inputEl: HTMLInputElement | undefined = $state();

  onMount(() => { tick().then(() => inputEl?.focus()); });

  function runSearch(): void {
    if (!query) { results = []; error = null; return; }
    try {
      const regex = new RegExp(query, 'gi');
      error = null;
      const matches: SearchResult[] = [];
      const allBuffers = bufferMap.getAll();
      for (const [file, content] of Object.entries(allBuffers)) {
        const lines = content.split('\n');
        for (let i = 0; i < lines.length; i++) {
          if (regex.test(lines[i])) {
            matches.push({ file, line: i + 1, content: lines[i].trim().slice(0, 80) });
            regex.lastIndex = 0;
          }
          if (matches.length >= 200) break;
        }
        if (matches.length >= 200) break;
      }
      results = matches;
      selectedIndex = 0;
    } catch (e) {
      error = 'Invalid regex pattern';
      results = [];
    }
  }

  function handleInput(e: Event): void {
    query = (e.target as HTMLInputElement).value;
    runSearch();
  }

  function navigateToResult(ref: SearchResult): void {
    navigationBroker.navigate({ targetViewId: 'urd.codeEditor', params: { path: ref.file, line: ref.line } });
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedIndex = Math.min(selectedIndex + 1, results.length - 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); selectedIndex = Math.max(selectedIndex - 1, 0); }
    else if (e.key === 'Enter' && results[selectedIndex]) { e.preventDefault(); navigateToResult(results[selectedIndex]); }
  }
</script>

<div class="forge-regex-search">
  <div class="forge-regex-search__header">
    <input bind:this={inputEl} class="forge-regex-search__input" type="text" value={query} placeholder="Regex pattern\u2026" oninput={handleInput} onkeydown={handleKeydown} spellcheck="false" autocomplete="off" />
    {#if error}<span class="forge-regex-search__error">{error}</span>{/if}
  </div>
  <div class="forge-regex-search__results">
    {#if results.length === 0}
      <div class="forge-regex-search__empty">{query ? 'No matches' : 'Enter a regex pattern to search'}</div>
    {:else}
      {#each results as ref, i}
        <button class="forge-regex-search__result" class:forge-regex-search__result--selected={i === selectedIndex} onclick={() => navigateToResult(ref)} type="button">
          <span class="forge-regex-search__file">{ref.file}:{ref.line}</span>
          <span class="forge-regex-search__content">{ref.content}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .forge-regex-search { display: flex; flex-direction: column; width: 100%; height: 100%; overflow: hidden; font-family: var(--forge-font-family-ui); }
  .forge-regex-search__header { padding: var(--forge-space-md); background-color: var(--forge-bg-secondary); border-bottom: 1px solid var(--forge-border-zone); }
  .forge-regex-search__input { width: 100%; height: 26px; padding: 0 var(--forge-space-md); background-color: var(--forge-bg-tertiary); border: 1px solid var(--forge-border-zone); border-radius: var(--forge-radius-sm); color: var(--forge-text-primary); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-sm); outline: none; }
  .forge-regex-search__input:focus { border-color: var(--forge-accent-primary); }
  .forge-regex-search__error { display: block; margin-top: var(--forge-space-xs); color: #e94560; font-size: var(--forge-font-size-xs); }
  .forge-regex-search__results { flex: 1; min-height: 0; overflow-y: auto; }
  .forge-regex-search__empty { display: flex; align-items: center; justify-content: center; padding: var(--forge-space-xl); color: var(--forge-text-muted); font-size: var(--forge-font-size-sm); }
  .forge-regex-search__result { display: flex; align-items: center; gap: var(--forge-space-sm); width: 100%; height: 28px; padding: 0 var(--forge-space-md); background: none; border: none; cursor: pointer; font-family: var(--forge-font-family-ui); font-size: var(--forge-font-size-sm); color: var(--forge-text-primary); text-align: left; }
  .forge-regex-search__result:hover { background-color: var(--forge-table-row-hover); }
  .forge-regex-search__result--selected { background-color: var(--forge-accent-selection); }
  .forge-regex-search__file { flex-shrink: 0; color: var(--forge-text-muted); font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); min-width: 120px; }
  .forge-regex-search__content { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: var(--forge-font-family-mono); font-size: var(--forge-font-size-xs); }
</style>
