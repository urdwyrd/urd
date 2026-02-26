<script lang="ts">
  /**
   * Global status bar — always-visible bottom bar.
   * Displays compiler status, active file, selection hint, stats, theme toggle, version.
   */

  import { getCurrentTheme, toggleTheme } from '../theme/ThemeEngine';
  import { subscribeToBus } from '../bus/useBusChannel';
  import { onMount } from 'svelte';

  let theme = $state(getCurrentTheme());
  let compilerStatus = $state('idle');
  let selectionHint = $state<string | null>(null);

  onMount(() => {
    const unsubs: (() => void)[] = [];

    unsubs.push(subscribeToBus('theme.changed', (payload: unknown) => {
      const p = payload as { theme: string };
      theme = p.theme as 'gloaming' | 'parchment';
    }));

    return () => unsubs.forEach((u) => u());
  });

  function handleToggleTheme() {
    toggleTheme();
  }
</script>

<div class="forge-status-bar">
  <div class="forge-status-bar__left">
    <span class="forge-status-bar__item forge-status-bar__compiler">
      {compilerStatus === 'idle' ? '◆ Ready' : compilerStatus}
    </span>
    {#if selectionHint}
      <span class="forge-status-bar__item">{selectionHint}</span>
    {/if}
  </div>

  <div class="forge-status-bar__right">
    <button
      class="forge-status-bar__btn"
      onclick={handleToggleTheme}
      title="Toggle Theme"
    >
      {theme === 'gloaming' ? '◑' : '◐'}
    </button>
    <span class="forge-status-bar__item forge-status-bar__version">
      Urd Forge v0.0.1
    </span>
  </div>
</div>

<style>
  .forge-status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    padding: 0 var(--forge-space-md);
    background: var(--forge-bg-secondary);
    border-top: 1px solid var(--forge-border-zone);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
    flex-shrink: 0;
  }

  .forge-status-bar__left,
  .forge-status-bar__right {
    display: flex;
    align-items: center;
    gap: var(--forge-space-lg);
  }

  .forge-status-bar__item {
    white-space: nowrap;
  }

  .forge-status-bar__compiler {
    color: var(--forge-status-success);
  }

  .forge-status-bar__version {
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-mono);
  }

  .forge-status-bar__btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-secondary);
    font-size: 14px;
    cursor: pointer;
  }

  .forge-status-bar__btn:hover {
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
  }
</style>
