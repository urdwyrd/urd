<script lang="ts">
  /**
   * Zone header — type selector dropdown + zone-specific toolbar slot.
   */

  import { mount, unmount, onDestroy } from 'svelte';
  import { viewRegistry } from '../views/ViewRegistry';
  import ViewTypeSelector from './ViewTypeSelector.svelte';

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
  let selectorContainer: HTMLDivElement | null = null;
  let selectorInstance: Record<string, any> | null = null;

  let currentView = $derived(viewRegistry.get(zoneTypeId));

  function closeSelector() {
    if (selectorInstance) {
      unmount(selectorInstance);
      selectorInstance = null;
    }
    if (selectorContainer) {
      selectorContainer.remove();
      selectorContainer = null;
    }
    dropdownOpen = false;
  }

  function openSelector(e: MouseEvent) {
    if (dropdownOpen) {
      closeSelector();
      return;
    }

    // Walk to the button element (e.target may be a child span)
    const btn = (e.target as HTMLElement).closest('.forge-zone-header__type-btn');
    if (!btn) return;
    const rect = btn.getBoundingClientRect();

    // Position container directly on document.body with fixed positioning.
    selectorContainer = document.createElement('div');
    selectorContainer.style.position = 'fixed';
    selectorContainer.style.zIndex = '9999';
    const flyoutWidth = 344; // 140 + 200 + border/padding
    const clampedLeft = Math.min(rect.left, window.innerWidth - flyoutWidth - 8);
    selectorContainer.style.left = `${Math.max(4, clampedLeft)}px`;

    const spaceBelow = window.innerHeight - rect.bottom;
    if (spaceBelow >= 200) {
      // Normal case: flyout below the button
      selectorContainer.style.top = `${rect.bottom + 2}px`;
      selectorContainer.style.maxHeight = `${spaceBelow - 10}px`;
    } else {
      // Edge case: flyout above the button (anchored at bottom, grows upward)
      selectorContainer.style.bottom = `${window.innerHeight - rect.top + 2}px`;
      selectorContainer.style.maxHeight = `${rect.top - 10}px`;
    }
    document.body.appendChild(selectorContainer);

    selectorInstance = mount(ViewTypeSelector, {
      target: selectorContainer,
      props: {
        zoneId,
        zoneTypeId,
        projectOpen,
        onSelect: (typeId: string) => {
          closeSelector();
          if (typeId !== zoneTypeId) {
            onChangeType(typeId);
          }
        },
        onClose: closeSelector,
      },
    });

    dropdownOpen = true;
  }

  onDestroy(() => {
    closeSelector();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="forge-zone-header"
  oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e); }}
>
  <div class="forge-zone-header__selector">
    <button
      class="forge-zone-header__type-btn"
      onclick={openSelector}
    >
      <span class="forge-zone-header__name">{currentView?.name ?? zoneTypeId}</span>
      <span class="forge-zone-header__arrow">▾</span>
    </button>
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
