<script lang="ts">
  /**
   * EntityRow — expandable entity detail row within the Entity Atlas.
   * L0: @id, type, location, inline key:value properties.
   * L1: properties table, containment, references, rules.
   */

  import type { UrdWorld, UrdEntity, PropertyDependencyEntry, FactSet, SymbolTableEntry } from '$lib/app/compiler/types';

  interface Props {
    entity: UrdEntity;
    world: UrdWorld;
    symbolEntry: SymbolTableEntry | undefined;
    locationName: string | null;
    propertyDeps: PropertyDependencyEntry[];
    factSet: FactSet | null;
    expanded: boolean;
    onToggle: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
  }

  let { entity, world, symbolEntry, locationName, propertyDeps, factSet, expanded, onToggle, onNavigate }: Props = $props();

  let inlineProps = $derived.by(() => {
    const entries = Object.entries(entity.properties);
    if (entries.length === 0) return '';
    return entries.map(([k, v]) => `${k}:${v}`).join('  ');
  });

  // Build a map of property name → dependency entry for this entity type
  let propDepMap = $derived.by(() => {
    const map = new Map<string, PropertyDependencyEntry>();
    for (const dep of propertyDeps) {
      map.set(dep.property, dep);
    }
    return map;
  });

  // Get type definition for property metadata
  let typeDef = $derived(entity.type ? world.types?.[entity.type] : null);

  // Find rules affecting this entity type
  let affectingRules = $derived.by(() => {
    if (!factSet || !entity.type) return [];
    return factSet.rules.filter((rule) => {
      const condReads = rule.condition_reads.map((i) => factSet.reads[i]).filter(Boolean);
      const effectWrites = rule.effect_writes.map((i) => factSet.writes[i]).filter(Boolean);
      return (
        condReads.some((r) => r.entity_type === entity.type) ||
        effectWrites.some((w) => w.entity_type === entity.type)
      );
    });
  });

  function handleToggle(): void {
    onToggle(`entity:${entity.id}`);
  }

  function goToSource(): void {
    if (symbolEntry?.file) {
      onNavigate(symbolEntry.file, symbolEntry.line);
    }
  }
</script>

<div class="forge-entity-row">
  <div class="forge-entity-row__header" role="button" tabindex="0" onclick={handleToggle} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleToggle(); } }}>
    <span class="forge-entity-row__toggle">{expanded ? '\u25BE' : '\u25B8'}</span>
    <button class="forge-entity-row__id" onclick={(e) => { e.stopPropagation(); goToSource(); }}>@{entity.id}</button>
    <span class="forge-entity-row__type">{entity.type ?? ''}</span>
    {#if locationName}
      <span class="forge-entity-row__location">{locationName}</span>
    {/if}
    <span class="forge-entity-row__inline-props">{inlineProps}</span>
  </div>

  {#if expanded}
    <div class="forge-entity-row__detail">
      <!-- Properties table -->
      {#if Object.keys(entity.properties).length > 0}
        <div class="forge-entity-row__sub-header">PROPERTIES</div>
        <div class="forge-entity-row__prop-table">
          {#each Object.entries(entity.properties) as [propName, propValue]}
            {@const dep = propDepMap.get(propName)}
            {@const propDef = typeDef?.properties?.[propName]}
            <div class="forge-entity-row__prop-row">
              <span class="forge-entity-row__prop-name">{propName}</span>
              {#if propDef?.type}
                <span class="forge-entity-row__prop-type">{propDef.type}</span>
              {/if}
              <span class="forge-entity-row__prop-value">{String(propValue)}</span>
              {#if dep}
                <span class="forge-entity-row__badge forge-entity-row__badge--read">{dep.read_count}R</span>
                <span class="forge-entity-row__badge forge-entity-row__badge--write">{dep.write_count}W</span>
                {#if dep.orphaned}
                  <span class="forge-entity-row__badge forge-entity-row__badge--orphan">{dep.orphaned}</span>
                {/if}
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <!-- Containment -->
      {#if entity.contains && entity.contains.length > 0}
        <div class="forge-entity-row__sub-header">CONTAINS</div>
        <div class="forge-entity-row__contains-list">
          {#each entity.contains as containedId}
            <span class="forge-entity-row__contained-id">@{containedId}</span>
          {/each}
        </div>
      {/if}

      <!-- Affecting rules -->
      {#if affectingRules.length > 0}
        <div class="forge-entity-row__sub-header">RULES ({affectingRules.length})</div>
        <div class="forge-entity-row__rules-list">
          {#each affectingRules as rule}
            <button
              class="forge-entity-row__rule-link"
              onclick={() => rule.span?.file && onNavigate(rule.span.file, rule.span.start_line)}
            >
              {rule.rule_id}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-entity-row {
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-entity-row__header {
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

  .forge-entity-row__header:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-entity-row__toggle {
    flex-shrink: 0;
    width: 10px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-entity-row__id {
    background: none;
    border: none;
    padding: 0;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-status-warning);
    cursor: pointer;
    flex-shrink: 0;
  }

  .forge-entity-row__id:hover {
    text-decoration: underline;
  }

  .forge-entity-row__type {
    color: var(--forge-text-muted);
    flex-shrink: 0;
  }

  .forge-entity-row__location {
    color: var(--forge-text-secondary);
    flex-shrink: 0;
  }

  .forge-entity-row__inline-props {
    color: var(--forge-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .forge-entity-row__detail {
    padding: var(--forge-space-xs) var(--forge-space-sm) var(--forge-space-sm);
    padding-left: calc(var(--forge-space-sm) + 16px);
  }

  .forge-entity-row__sub-header {
    font-family: var(--forge-font-family-ui);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--forge-text-muted);
    margin-top: var(--forge-space-xs);
    margin-bottom: 2px;
  }

  .forge-entity-row__prop-table {
    display: flex;
    flex-direction: column;
  }

  .forge-entity-row__prop-row {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    height: 20px;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-entity-row__prop-name {
    color: var(--forge-text-secondary);
    min-width: 80px;
  }

  .forge-entity-row__prop-type {
    color: var(--forge-text-muted);
    font-size: 10px;
    min-width: 50px;
  }

  .forge-entity-row__prop-value {
    color: var(--forge-text-primary);
  }

  .forge-entity-row__badge {
    font-size: 10px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    line-height: 14px;
  }

  .forge-entity-row__badge--read {
    background-color: color-mix(in srgb, var(--forge-status-success, #4ade80) 15%, transparent);
    color: var(--forge-status-success, #4ade80);
  }

  .forge-entity-row__badge--write {
    background-color: color-mix(in srgb, var(--forge-status-warning) 15%, transparent);
    color: var(--forge-status-warning);
  }

  .forge-entity-row__badge--orphan {
    background-color: color-mix(in srgb, var(--forge-status-warning) 15%, transparent);
    color: var(--forge-status-warning);
  }

  .forge-entity-row__contains-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--forge-space-xs);
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-entity-row__contained-id {
    color: var(--forge-status-warning);
  }

  .forge-entity-row__rules-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--forge-space-xs);
  }

  .forge-entity-row__rule-link {
    background: none;
    border: none;
    padding: 0;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
    cursor: pointer;
  }

  .forge-entity-row__rule-link:hover {
    text-decoration: underline;
    color: var(--forge-text-primary);
  }
</style>
