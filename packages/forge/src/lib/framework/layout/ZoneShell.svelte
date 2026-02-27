<script lang="ts">
  /**
   * ZoneShell — header bar + viewport wrapper for a single zone.
   * Hosts the view component, wrapping it in an error boundary.
   */

  import ZoneHeader from './ZoneHeader.svelte';
  import ZoneErrorBoundary from './ZoneErrorBoundary.svelte';
  import ZoneLoadingState from './ZoneLoadingState.svelte';
  import CornerHotspot from './CornerHotspot.svelte';
  import { viewRegistry } from '../views/ViewRegistry';
  import { focusService } from '../focus/FocusService.svelte';
  import type { Component } from 'svelte';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    zoneState: unknown;
    projectOpen: boolean;
    onChangeType: (typeId: string) => void;
    onSplit: (direction: 'horizontal' | 'vertical') => void;
    onStateChange: (state: unknown) => void;
    onContextMenu: (e: MouseEvent) => void;
  }

  let {
    zoneId,
    zoneTypeId,
    zoneState,
    projectOpen,
    onChangeType,
    onSplit,
    onStateChange,
    onContextMenu,
  }: Props = $props();

  let loadedComponent = $state<Component | null>(null);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let reloadKey = $state(0);
  let isSingletonBlocked = $state(false);

  // Singleton lifecycle — claim/release ownership
  $effect(() => {
    const typeId = zoneTypeId;
    const id = zoneId;
    isSingletonBlocked = false;

    if (viewRegistry.isSingleton(typeId)) {
      if (viewRegistry.isSingletonActive(typeId) && viewRegistry.getSingletonZoneId(typeId) !== id) {
        // Another zone owns this singleton — show placeholder
        isSingletonBlocked = true;
        loading = false;
        loadedComponent = null;
        loadError = null;
        return;
      }
      // Claim ownership
      viewRegistry.markSingletonActive(typeId, id);
    }

    return () => {
      // Release ownership when zone type changes or component unmounts
      if (viewRegistry.isSingleton(typeId) && viewRegistry.getSingletonZoneId(typeId) === id) {
        viewRegistry.markSingletonInactive(typeId);
      }
    };
  });

  function focusOwner(): void {
    const ownerZoneId = viewRegistry.getSingletonZoneId(zoneTypeId);
    if (ownerZoneId) {
      focusService.focusZone(ownerZoneId, zoneTypeId);
    }
  }

  // Load the view component lazily
  $effect(() => {
    // Re-run when zoneTypeId or reloadKey changes
    const typeId = zoneTypeId;
    const _key = reloadKey;

    // Skip loading if this zone is blocked by singleton ownership
    if (isSingletonBlocked) return;

    loading = true;
    loadError = null;
    loadedComponent = null;

    const view = viewRegistry.get(typeId);
    if (!view) {
      loading = false;
      loadError = `Unknown view type: ${typeId}`;
      return;
    }

    view.component().then((mod) => {
      loadedComponent = mod.default;
      loading = false;
    }).catch((err) => {
      loadError = err instanceof Error ? err.message : String(err);
      loading = false;
    });
  });

  function handleReload() {
    reloadKey++;
  }

  function handleHeaderContextMenu(e: MouseEvent) {
    onContextMenu(e);
  }
</script>

<div class="forge-zone-shell" data-zone-id={zoneId}>
  <ZoneHeader
    {zoneId}
    {zoneTypeId}
    {projectOpen}
    {onChangeType}
    {onSplit}
    onContextMenu={handleHeaderContextMenu}
  />

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="forge-zone-viewport" oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e); }}>
    <CornerHotspot {onSplit} />
    {#if isSingletonBlocked}
      <div class="forge-zone-viewport__singleton-placeholder">
        <p>Already visible in another panel</p>
        <button class="forge-zone-viewport__focus-btn" onclick={focusOwner}>Focus</button>
      </div>
    {:else}
      <ZoneErrorBoundary
        {zoneId}
        {zoneTypeId}
        onReload={handleReload}
        {onChangeType}
      >
        {#if loading}
          <ZoneLoadingState />
        {:else if loadError}
          <div class="forge-zone-viewport__error">
            <p>Failed to load view: {loadError}</p>
          </div>
        {:else if loadedComponent}
          {@const ViewComponent = loadedComponent}
          <ViewComponent
            {zoneId}
            {zoneTypeId}
            state={zoneState}
            {onStateChange}
          />
        {/if}
      </ZoneErrorBoundary>
    {/if}
  </div>
</div>

<style>
  .forge-zone-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    border: 1px solid var(--forge-border-zone);
  }

  .forge-zone-viewport {
    flex: 1;
    overflow: auto;
    background: var(--forge-bg-zone-viewport);
    position: relative;
  }

  .forge-zone-viewport__error {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--forge-status-error);
    font-size: var(--forge-font-size-sm);
    padding: var(--forge-space-xl);
  }

  .forge-zone-viewport__singleton-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--forge-space-md);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-zone-viewport__focus-btn {
    padding: var(--forge-space-xs) var(--forge-space-md);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-secondary);
    color: var(--forge-text-primary);
    font-size: var(--forge-font-size-sm);
    cursor: pointer;
  }

  .forge-zone-viewport__focus-btn:hover {
    background: var(--forge-bg-tertiary);
  }
</style>
