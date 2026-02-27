<script lang="ts">
  /**
   * ViewTypeSelector — two-column categorized flyout for switching zone view types.
   * Left column shows categories, right column shows views in the hovered category.
   * Rendered with position:fixed so it's never clipped by small panels.
   */

  import { viewRegistry } from '../views/ViewRegistry';
  import { onMount } from 'svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    projectOpen: boolean;
    onSelect: (typeId: string) => void;
    onClose: () => void;
  }

  let { zoneId, zoneTypeId, projectOpen, onSelect, onClose }: Props = $props();

  let panelEl: HTMLDivElement | undefined = $state();

  // Group available views by category, sorted alphabetically
  let viewsByCategory = $derived.by(() => {
    const available = viewRegistry.listAvailable(projectOpen);
    const grouped = new Map<string, typeof available>();
    for (const v of available) {
      const list = grouped.get(v.category) ?? [];
      list.push(v);
      grouped.set(v.category, list);
    }
    // Sort views within each category
    for (const list of grouped.values()) {
      list.sort((a, b) => a.name.localeCompare(b.name));
    }
    // Return sorted category entries
    return [...grouped.entries()].sort(([a], [b]) => a.localeCompare(b));
  });

  // Initialise active category to the current view's category
  let currentViewCategory = $derived(viewRegistry.get(zoneTypeId)?.category ?? '');
  let activeCategory = $state('');

  // Set initial category on mount
  $effect(() => {
    if (!activeCategory && currentViewCategory) {
      activeCategory = currentViewCategory;
    }
  });

  // Views in the active category
  let activeCategoryViews = $derived(
    viewsByCategory.find(([cat]) => cat === activeCategory)?.[1] ?? []
  );

  // Positioning is handled by the parent container div.
  onMount(() => {

    const handleMouseDown = (e: MouseEvent) => {
      if (panelEl && !panelEl.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };

    // Delay to avoid immediate close from the triggering click
    requestAnimationFrame(() => {
      document.addEventListener('mousedown', handleMouseDown);
      document.addEventListener('keydown', handleKeyDown);
    });

    return () => {
      document.removeEventListener('mousedown', handleMouseDown);
      document.removeEventListener('keydown', handleKeyDown);
    };
  });

  function handleViewClick(typeId: string) {
    onSelect(typeId);
  }
</script>

<div class="forge-view-selector" bind:this={panelEl} role="menu">
  <div class="forge-view-selector__categories">
    {#each viewsByCategory as [category]}
      <button
        class="forge-view-selector__category"
        class:forge-view-selector__category--active={category === activeCategory}
        onpointerenter={() => { activeCategory = category; }}
        onclick={() => { activeCategory = category; }}
        role="menuitem"
      >
        {category}
      </button>
    {/each}
  </div>

  <div class="forge-view-selector__views">
    {#each activeCategoryViews as view (view.id)}
      {@const isDisabledSingleton =
        viewRegistry.isSingleton(view.id) &&
        viewRegistry.isSingletonActive(view.id) &&
        viewRegistry.getSingletonZoneId(view.id) !== zoneId}
      <button
        class="forge-view-selector__view"
        class:forge-view-selector__view--active={view.id === zoneTypeId}
        class:forge-view-selector__view--disabled={isDisabledSingleton}
        disabled={isDisabledSingleton}
        title={isDisabledSingleton ? 'Already visible in another panel' : ''}
        onclick={() => handleViewClick(view.id)}
        role="menuitem"
      >
        {view.name}
      </button>
    {/each}
  </div>
</div>

<style>
  .forge-view-selector {
    display: flex;
    max-height: inherit;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    overflow: hidden;
  }

  .forge-view-selector__categories {
    display: flex;
    flex-direction: column;
    width: 140px;
    border-right: 1px solid var(--forge-border-zone);
    overflow-y: auto;
    padding: var(--forge-space-xs) 0;
    flex-shrink: 0;
  }

  .forge-view-selector__category {
    display: block;
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-md);
    border: none;
    background: transparent;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
  }

  .forge-view-selector__category:hover {
    background: var(--forge-bg-tertiary);
  }

  .forge-view-selector__category--active {
    background: var(--forge-bg-tertiary);
    border-left: 2px solid var(--forge-accent-primary);
    padding-left: calc(var(--forge-space-md) - 2px);
  }

  .forge-view-selector__views {
    display: flex;
    flex-direction: column;
    width: 200px;
    overflow-y: auto;
    padding: var(--forge-space-xs) 0;
  }

  .forge-view-selector__view {
    display: block;
    width: 100%;
    padding: var(--forge-space-sm) var(--forge-space-md);
    border: none;
    background: transparent;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
  }

  .forge-view-selector__view:hover:not(:disabled) {
    background: var(--forge-bg-tertiary);
  }

  .forge-view-selector__view--active {
    color: var(--forge-accent-primary);
  }

  .forge-view-selector__view--disabled {
    color: var(--forge-text-muted);
    cursor: default;
  }
</style>
