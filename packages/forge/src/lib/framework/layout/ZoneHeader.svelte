<script lang="ts">
  /**
   * Zone header — type selector dropdown + zone-specific toolbar slot.
   */

  import { viewRegistry } from '../views/ViewRegistry';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    projectOpen: boolean;
    onChangeType: (typeId: string) => void;
    onSplit: (direction: 'horizontal' | 'vertical') => void;
    onContextMenu: (e: MouseEvent) => void;
  }

  let { zoneId, zoneTypeId, projectOpen, onChangeType, onSplit, onContextMenu }: Props = $props();

  let dropdownOpen = $state(false);

  let currentView = $derived(viewRegistry.get(zoneTypeId));
  let availableViews = $derived(viewRegistry.listAvailable(projectOpen));
  let viewsByCategory = $derived(() => {
    const grouped = new Map<string, typeof availableViews>();
    for (const v of availableViews) {
      const list = grouped.get(v.category) ?? [];
      list.push(v);
      grouped.set(v.category, list);
    }
    return grouped;
  });

  function selectView(typeId: string) {
    dropdownOpen = false;
    if (typeId !== zoneTypeId) {
      onChangeType(typeId);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      dropdownOpen = false;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="forge-zone-header"
  oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e); }}
>
  <div class="forge-zone-header__selector">
    <button
      class="forge-zone-header__type-btn"
      onclick={() => { dropdownOpen = !dropdownOpen; }}
      onkeydown={handleKeyDown}
    >
      <span class="forge-zone-header__name">{currentView?.name ?? zoneTypeId}</span>
      <span class="forge-zone-header__arrow">▾</span>
    </button>

    {#if dropdownOpen}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="forge-zone-header__dropdown" onkeydown={handleKeyDown}>
        {#each [...viewsByCategory()] as [category, views]}
          <div class="forge-zone-header__category">{category}</div>
          {#each views as view}
            {@const isDisabledSingleton =
              viewRegistry.isSingleton(view.id) &&
              viewRegistry.isSingletonActive(view.id) &&
              viewRegistry.getSingletonZoneId(view.id) !== zoneId}
            <button
              class="forge-zone-header__option"
              class:forge-zone-header__option--active={view.id === zoneTypeId}
              class:forge-zone-header__option--disabled={isDisabledSingleton}
              disabled={isDisabledSingleton}
              title={isDisabledSingleton ? 'Already visible in another panel' : ''}
              onclick={() => selectView(view.id)}
            >
              {view.name}
            </button>
          {/each}
        {/each}
      </div>
    {/if}
  </div>

  <div class="forge-zone-header__actions">
    <button class="forge-zone-header__action-btn" onclick={() => onSplit('horizontal')} title="Split Left / Right">
      ⬌
    </button>
    <button class="forge-zone-header__action-btn" onclick={() => onSplit('vertical')} title="Split Top / Bottom">
      ⬍
    </button>
  </div>
</div>

<style>
  .forge-zone-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 var(--forge-space-sm);
    background: var(--forge-bg-zone-header);
    border-bottom: 1px solid var(--forge-border-zone);
    font-size: var(--forge-font-size-sm);
    flex-shrink: 0;
  }

  .forge-zone-header__selector {
    position: relative;
  }

  .forge-zone-header__type-btn {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
  }

  .forge-zone-header__type-btn:hover {
    background: var(--forge-bg-secondary);
  }

  .forge-zone-header__arrow {
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-zone-header__dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    min-width: 180px;
    max-height: 300px;
    overflow-y: auto;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: var(--forge-z-dropdown);
    padding: var(--forge-space-xs) 0;
  }

  .forge-zone-header__category {
    padding: var(--forge-space-xs) var(--forge-space-md);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-zone-header__option {
    display: block;
    width: 100%;
    padding: var(--forge-space-xs) var(--forge-space-md);
    border: none;
    background: transparent;
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    text-align: left;
    cursor: pointer;
  }

  .forge-zone-header__option:hover {
    background: var(--forge-bg-tertiary);
  }

  .forge-zone-header__option--active {
    color: var(--forge-accent-primary);
  }

  .forge-zone-header__option--disabled {
    color: var(--forge-text-muted);
    cursor: default;
  }

  .forge-zone-header__option--disabled:hover {
    background: transparent;
  }

  .forge-zone-header__actions {
    display: flex;
    gap: 2px;
  }

  .forge-zone-header__action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    border-radius: var(--forge-radius-sm);
    background: transparent;
    color: var(--forge-text-muted);
    font-size: 12px;
    cursor: pointer;
  }

  .forge-zone-header__action-btn:hover {
    background: var(--forge-bg-secondary);
    color: var(--forge-text-primary);
  }
</style>
