<script lang="ts">
  /**
   * PropertyFlowRow — expandable property row with read/write sites.
   * L0: Type.property, read count, write count, orphan badge.
   * L1: individual read/write site details.
   */

  import type { PropertyDependencyEntry, FactSet } from '$lib/app/compiler/types';

  interface Props {
    property: PropertyDependencyEntry;
    factSet: FactSet;
    expanded: boolean;
    onToggle: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
  }

  let { property, factSet, expanded, onToggle, onNavigate }: Props = $props();

  let propKey = $derived(`${property.entity_type}.${property.property}`);

  let readSites = $derived.by(() => {
    return property.read_indices
      .map((i) => factSet.reads[i])
      .filter(Boolean);
  });

  let writeSites = $derived.by(() => {
    return property.write_indices
      .map((i) => factSet.writes[i])
      .filter(Boolean);
  });

  function handleToggle(): void {
    onToggle(`prop:${propKey}`);
  }
</script>

<div class="forge-property-flow-row">
  <button class="forge-property-flow-row__header" onclick={handleToggle}>
    <span class="forge-property-flow-row__toggle">{expanded ? '\u25BE' : '\u25B8'}</span>
    <span class="forge-property-flow-row__key">{propKey}</span>
    <span class="forge-property-flow-row__badge forge-property-flow-row__badge--read">
      {property.read_count}R
    </span>
    <span class="forge-property-flow-row__badge forge-property-flow-row__badge--write">
      {property.write_count}W
    </span>
    {#if property.orphaned}
      <span class="forge-property-flow-row__badge forge-property-flow-row__badge--orphan">
        {property.orphaned}
      </span>
    {/if}
  </button>

  {#if expanded}
    <div class="forge-property-flow-row__detail">
      {#if readSites.length > 0}
        <div class="forge-property-flow-row__sub-header">READS</div>
        {#each readSites as read}
          <button
            class="forge-property-flow-row__site"
            onclick={() => onNavigate(read.span.file, read.span.start_line)}
          >
            <span class="forge-property-flow-row__site-kind">{read.site.kind}: {read.site.id}</span>
            <span class="forge-property-flow-row__site-op">{read.operator} {read.value_literal}</span>
            <span class="forge-property-flow-row__site-loc">{read.span.file}:{read.span.start_line}</span>
          </button>
        {/each}
      {/if}

      {#if writeSites.length > 0}
        <div class="forge-property-flow-row__sub-header">WRITES</div>
        {#each writeSites as write}
          <button
            class="forge-property-flow-row__site"
            onclick={() => onNavigate(write.span.file, write.span.start_line)}
          >
            <span class="forge-property-flow-row__site-kind">{write.site.kind}: {write.site.id}</span>
            <span class="forge-property-flow-row__site-op">{write.operator} {write.value_expr}</span>
            <span class="forge-property-flow-row__site-loc">{write.span.file}:{write.span.start_line}</span>
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-property-flow-row {
    border-bottom: 1px solid color-mix(in srgb, var(--forge-border-zone) 50%, transparent);
  }

  .forge-property-flow-row__header {
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

  .forge-property-flow-row__header:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-property-flow-row__toggle {
    flex-shrink: 0;
    width: 10px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-property-flow-row__key {
    color: var(--forge-status-warning);
    flex: 1;
  }

  .forge-property-flow-row__badge {
    font-size: 10px;
    padding: 0 3px;
    border-radius: var(--forge-radius-sm);
    line-height: 14px;
    flex-shrink: 0;
  }

  .forge-property-flow-row__badge--read {
    background-color: color-mix(in srgb, var(--forge-status-success, #4ade80) 15%, transparent);
    color: var(--forge-status-success, #4ade80);
  }

  .forge-property-flow-row__badge--write {
    background-color: color-mix(in srgb, var(--forge-status-warning) 15%, transparent);
    color: var(--forge-status-warning);
  }

  .forge-property-flow-row__badge--orphan {
    background-color: color-mix(in srgb, var(--forge-status-warning) 15%, transparent);
    color: var(--forge-status-warning);
  }

  .forge-property-flow-row__detail {
    padding: var(--forge-space-xs) var(--forge-space-sm) var(--forge-space-sm);
    padding-left: calc(var(--forge-space-sm) + 16px);
  }

  .forge-property-flow-row__sub-header {
    font-family: var(--forge-font-family-ui);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--forge-text-muted);
    margin-top: var(--forge-space-xs);
    margin-bottom: 2px;
  }

  .forge-property-flow-row__site {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    width: 100%;
    height: 20px;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-text-secondary);
  }

  .forge-property-flow-row__site:hover {
    color: var(--forge-text-primary);
  }

  .forge-property-flow-row__site-kind {
    min-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forge-property-flow-row__site-op {
    color: var(--forge-text-primary);
    min-width: 80px;
  }

  .forge-property-flow-row__site-loc {
    color: var(--forge-text-muted);
    font-size: 10px;
    margin-left: auto;
  }
</style>
