<script lang="ts">
  /**
   * ZoneShell — header bar + viewport wrapper for a single zone.
   * Hosts the view component, wrapping it in an error boundary.
   */

  import ZoneHeader from './ZoneHeader.svelte';
  import ZoneErrorBoundary from './ZoneErrorBoundary.svelte';
  import ZoneLoadingState from './ZoneLoadingState.svelte';
  import { viewRegistry } from '../views/ViewRegistry';
  import { bus } from '../bus/MessageBus';
  import type { Component } from 'svelte';
  import { onMount } from 'svelte';

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

  // Load the view component lazily
  $effect(() => {
    // Re-run when zoneTypeId or reloadKey changes
    const typeId = zoneTypeId;
    const _key = reloadKey;
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
    -webkit-user-select: contain;
    user-select: contain;
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
</style>
