<script lang="ts">
  /**
   * LocationRow — expandable location detail row within LocationMap.
   * L0: location name, exit count, entity count, start badge, source ref.
   * L1: exits, entities, containment.
   */

  import type { UrdLocation, UrdEntity, SymbolTableEntry } from '$lib/app/compiler/types';

  interface Props {
    location: UrdLocation;
    entities: UrdEntity[];
    isStart: boolean;
    symbolEntry: SymbolTableEntry | undefined;
    expanded: boolean;
    onToggle: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
  }

  let { location, entities, isStart, symbolEntry, expanded, onToggle, onNavigate }: Props = $props();

  function handleToggle(): void {
    onToggle(`location:${location.id}`);
  }

  function goToSource(): void {
    if (symbolEntry?.file) {
      onNavigate(symbolEntry.file, symbolEntry.line);
    }
  }
</script>

<div class="forge-location-row">
  <button class="forge-location-row__header" onclick={handleToggle}>
    <span class="forge-location-row__toggle">{expanded ? '\u25BE' : '\u25B8'}</span>
    <span class="forge-location-row__name">{location.name}</span>
    <span class="forge-location-row__badge">{location.exits.length} exits</span>
    <span class="forge-location-row__badge">{entities.length} entities</span>
    {#if isStart}
      <span class="forge-location-row__badge forge-location-row__badge--start">\u25B6 start</span>
    {/if}
    {#if symbolEntry}
      <span class="forge-location-row__source">{symbolEntry.file}:{symbolEntry.line}</span>
    {/if}
  </button>

  {#if expanded}
    <div class="forge-location-row__detail">
      {#if location.exits.length > 0}
        <div class="forge-location-row__sub-header">EXITS</div>
        {#each location.exits as exit}
          <div class="forge-location-row__exit">
            <span class="forge-location-row__exit-arrow">\u2192</span>
            <span class="forge-location-row__exit-target">{exit.target}</span>
            <span class="forge-location-row__exit-dir">via {exit.direction}</span>
            {#if exit.condition}
              <span class="forge-location-row__exit-cond">? {exit.condition}</span>
            {:else}
              <span class="forge-location-row__exit-badge">unconditional</span>
            {/if}
          </div>
        {/each}
      {/if}

      {#if entities.length > 0}
        <div class="forge-location-row__sub-header">ENTITIES</div>
        {#each entities as entity}
          <div class="forge-location-row__entity">
            <span class="forge-location-row__entity-id">@{entity.id}</span>
            <span class="forge-location-row__entity-type">{entity.type ?? ''}</span>
            <span class="forge-location-row__entity-props">
              {Object.entries(entity.properties).map(([k, v]) => `${k}:${v}`).join('  ')}
            </span>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-location-row {
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-location-row__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    width: 100%;
    height: 24px;
    padding: 0 var(--forge-space-sm);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-primary);
  }

  .forge-location-row__header:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-location-row__toggle {
    flex-shrink: 0;
    width: 10px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-location-row__name {
    font-weight: 600;
    color: var(--forge-text-primary);
    flex: 1;
  }

  .forge-location-row__badge {
    font-size: 10px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
    color: var(--forge-text-muted);
    line-height: 14px;
    flex-shrink: 0;
  }

  .forge-location-row__badge--start {
    background-color: color-mix(in srgb, var(--forge-status-success, #4ade80) 15%, transparent);
    color: var(--forge-status-success, #4ade80);
  }

  .forge-location-row__source {
    color: var(--forge-text-muted);
    font-size: 10px;
    flex-shrink: 0;
  }

  .forge-location-row__detail {
    padding: var(--forge-space-xs) var(--forge-space-sm) var(--forge-space-sm);
    padding-left: calc(var(--forge-space-sm) + 16px);
  }

  .forge-location-row__sub-header {
    font-family: var(--forge-font-family-ui);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--forge-text-muted);
    margin-top: var(--forge-space-xs);
    margin-bottom: 2px;
  }

  .forge-location-row__exit {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    height: 20px;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
  }

  .forge-location-row__exit-arrow {
    color: var(--forge-status-success, #4ade80);
    flex-shrink: 0;
  }

  .forge-location-row__exit-target {
    color: var(--forge-text-primary);
  }

  .forge-location-row__exit-dir {
    color: var(--forge-text-muted);
  }

  .forge-location-row__exit-cond {
    color: var(--forge-status-warning);
    font-size: 10px;
  }

  .forge-location-row__exit-badge {
    font-size: 9px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 12%, transparent);
    color: var(--forge-text-muted);
    line-height: 14px;
  }

  .forge-location-row__entity {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    height: 20px;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-location-row__entity-id {
    color: var(--forge-status-warning);
  }

  .forge-location-row__entity-type {
    color: var(--forge-text-muted);
  }

  .forge-location-row__entity-props {
    color: var(--forge-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
</style>
