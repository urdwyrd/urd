<script lang="ts">
  /**
   * LocationMap — textual outline of locations with exits and entities (Section 5).
   */

  import type { UrdWorld, UrdEntity, SymbolTable, SymbolTableEntry } from '$lib/app/compiler/types';
  import LocationRow from './LocationRow.svelte';

  interface Props {
    world: UrdWorld;
    symbolMap: Map<string, SymbolTableEntry>;
    entityLocationMap: Map<string, string>;
    expandedRows: Record<string, boolean>;
    onToggleRow: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
    filterText: string;
  }

  let { world, symbolMap, entityLocationMap, expandedRows, onToggleRow, onNavigate, filterText }: Props = $props();

  // Build location → entities map
  let locationEntities = $derived.by(() => {
    const map = new Map<string, UrdEntity[]>();
    for (const entity of world.entities) {
      const locId = entityLocationMap.get(entity.id);
      if (locId) {
        const list = map.get(locId) ?? [];
        list.push(entity);
        map.set(locId, list);
      }
    }
    return map;
  });

  let filteredLocations = $derived.by(() => {
    if (!filterText) return world.locations;
    const q = filterText.toLowerCase();
    return world.locations.filter(
      (loc) =>
        loc.id.toLowerCase().includes(q) ||
        loc.name.toLowerCase().includes(q) ||
        loc.exits.some(
          (e) => e.direction.toLowerCase().includes(q) || e.target.toLowerCase().includes(q),
        ),
    );
  });
</script>

<div class="forge-location-map">
  {#if filteredLocations.length === 0}
    <div class="forge-location-map__empty">No locations</div>
  {:else}
    {#each filteredLocations as location}
      <LocationRow
        {location}
        entities={locationEntities.get(location.id) ?? []}
        isStart={world.world?.start === location.id || world.world?.start === location.name}
        symbolEntry={symbolMap.get(location.id) ?? symbolMap.get(location.name)}
        expanded={expandedRows[`location:${location.id}`] ?? false}
        onToggle={onToggleRow}
        {onNavigate}
      />
    {/each}
  {/if}
</div>

<style>
  .forge-location-map {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-location-map__empty {
    padding: var(--forge-space-md);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    text-align: center;
  }
</style>
