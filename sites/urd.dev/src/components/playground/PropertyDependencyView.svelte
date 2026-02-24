<script lang="ts">
  import type { PropertyIndex, PropertyEntry, FactSet } from './compiler-bridge';

  interface Props {
    propertyIndex: PropertyIndex;
    facts: FactSet;
    onDiagnosticClick?: (line: number, col: number) => void;
  }

  let { propertyIndex, facts, onDiagnosticClick }: Props = $props();

  type FilterMode = 'all' | 'read_only' | 'write_only';
  let filter: FilterMode = $state('all');
  let expanded: Record<string, boolean> = $state({});

  let filteredProperties = $derived(
    propertyIndex.properties.filter((p) => {
      if (filter === 'read_only') return p.orphaned === 'read_never_written';
      if (filter === 'write_only') return p.orphaned === 'written_never_read';
      return true;
    })
  );

  function propKey(p: PropertyEntry): string {
    return `${p.entity_type}.${p.property}`;
  }

  function toggle(key: string) {
    expanded[key] = !expanded[key];
  }

  function isOpen(key: string): boolean {
    return !!expanded[key];
  }

  function clickSpan(line: number, col: number) {
    onDiagnosticClick?.(line, col);
  }

  function siteLabel(site: { kind: string; id: string }): string {
    return `${site.kind}: ${site.id}`;
  }

  function firstLine(p: PropertyEntry): number | null {
    if (p.read_indices.length > 0) {
      return facts.reads[p.read_indices[0]]?.span.start_line ?? null;
    }
    if (p.write_indices.length > 0) {
      return facts.writes[p.write_indices[0]]?.span.start_line ?? null;
    }
    return null;
  }
</script>

<div class="pd-view">
  <!-- Summary -->
  <div class="pd-summary">
    <span class="pd-total">{propertyIndex.summary.total_properties} properties</span>
    <span class="pd-counts">
      {propertyIndex.summary.total_reads} reads
      · {propertyIndex.summary.total_writes} writes
      {#if propertyIndex.summary.read_never_written > 0}
        · <span class="pd-orphan-count">{propertyIndex.summary.read_never_written} read‑only</span>
      {/if}
      {#if propertyIndex.summary.written_never_read > 0}
        · <span class="pd-orphan-count">{propertyIndex.summary.written_never_read} write‑only</span>
      {/if}
    </span>
  </div>

  <!-- Filter buttons -->
  <div class="pd-filters" role="group" aria-label="Property filter">
    <button
      class="pd-filter"
      class:pd-filter-active={filter === 'all'}
      onclick={() => filter = 'all'}
    >All <span class="pd-filter-count">{propertyIndex.summary.total_properties}</span></button>
    <button
      class="pd-filter"
      class:pd-filter-active={filter === 'read_only'}
      onclick={() => filter = 'read_only'}
    >Read‑only <span class="pd-filter-count">{propertyIndex.summary.read_never_written}</span></button>
    <button
      class="pd-filter"
      class:pd-filter-active={filter === 'write_only'}
      onclick={() => filter = 'write_only'}
    >Write‑only <span class="pd-filter-count">{propertyIndex.summary.written_never_read}</span></button>
  </div>

  <!-- Property list -->
  {#each filteredProperties as prop (propKey(prop))}
    {@const key = propKey(prop)}
    {@const line = firstLine(prop)}
    <button
      class="pd-row"
      onclick={() => toggle(key)}
    >
      <span class="pd-toggle">{isOpen(key) ? '▾' : '▸'}</span>
      <span class="pd-prop">{key}</span>
      <span class="pd-badge pd-reads">{prop.read_count}R</span>
      <span class="pd-badge pd-writes">{prop.write_count}W</span>
      {#if prop.orphaned === 'read_never_written'}
        <span class="pd-badge pd-orphaned">read only</span>
      {:else if prop.orphaned === 'written_never_read'}
        <span class="pd-badge pd-orphaned">write only</span>
      {/if}
      {#if line}
        <span class="pd-line">L{line}</span>
      {/if}
    </button>

    {#if isOpen(key)}
      <div class="pd-detail">
        {#if prop.read_indices.length > 0}
          <div class="pd-detail-label">Reads</div>
          {#each prop.read_indices as idx}
            {@const read = facts.reads[idx]}
            {#if read}
              <button class="pd-site-row" onclick={() => clickSpan(read.span.start_line, read.span.start_col)}>
                <span class="pd-site-label">{siteLabel(read.site)}</span>
                <span class="pd-site-op">{read.operator} {read.value_literal}</span>
                <span class="pd-site-line">L{read.span.start_line}</span>
              </button>
            {/if}
          {/each}
        {/if}
        {#if prop.write_indices.length > 0}
          <div class="pd-detail-label">Writes</div>
          {#each prop.write_indices as idx}
            {@const write = facts.writes[idx]}
            {#if write}
              <button class="pd-site-row" onclick={() => clickSpan(write.span.start_line, write.span.start_col)}>
                <span class="pd-site-label">{siteLabel(write.site)}</span>
                <span class="pd-site-op">{write.operator} {write.value_expr}</span>
                <span class="pd-site-line">L{write.span.start_line}</span>
              </button>
            {/if}
          {/each}
        {/if}
      </div>
    {/if}
  {/each}

  {#if filteredProperties.length === 0}
    <div class="pd-empty">
      {#if filter !== 'all'}
        No {filter === 'read_only' ? 'read-only' : 'write-only'} properties.
      {:else}
        No properties extracted.
      {/if}
    </div>
  {/if}
</div>

<style>
  .pd-view {
    font-family: var(--mono);
    font-size: 12px;
  }

  /* --- Summary --- */

  .pd-summary {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    color: var(--dim);
    font-size: 11px;
  }

  .pd-total {
    color: var(--text);
    font-weight: 600;
  }

  .pd-counts {
    color: var(--faint);
  }

  .pd-orphan-count {
    color: var(--amber-light);
  }

  /* --- Filter buttons --- */

  .pd-filters {
    display: flex;
    gap: 0;
    padding: 0 8px;
    border-bottom: 1px solid var(--border);
  }

  .pd-filter {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 10px;
    border: none;
    background: none;
    color: var(--faint);
    font-family: var(--display);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
  }

  .pd-filter:hover {
    color: var(--dim);
  }

  .pd-filter-active {
    color: var(--text);
    border-bottom-color: var(--gold);
  }

  .pd-filter-count {
    color: var(--faint);
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 400;
  }

  .pd-filter-active .pd-filter-count {
    color: var(--text);
  }

  /* --- Property rows --- */

  .pd-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
    width: 100%;
    padding: 4px 12px 4px 12px;
    border: none;
    background: none;
    color: var(--dim);
    font-family: var(--mono);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .pd-row:hover {
    background: var(--surface);
  }

  .pd-toggle {
    color: var(--faint);
    width: 10px;
    flex-shrink: 0;
  }

  .pd-prop {
    color: var(--amber-light);
  }

  .pd-badge {
    font-size: 10px;
    padding: 0 4px;
    border-radius: 3px;
  }

  .pd-reads {
    background: color-mix(in srgb, var(--green-light) 15%, transparent);
    color: var(--green-light);
  }

  .pd-writes {
    background: color-mix(in srgb, var(--gold) 15%, transparent);
    color: var(--gold);
  }

  .pd-orphaned {
    background: color-mix(in srgb, var(--amber-light) 15%, transparent);
    color: var(--amber-light);
  }

  .pd-line {
    margin-left: auto;
    color: var(--faint);
    font-size: 10px;
  }

  /* --- Expanded detail --- */

  .pd-detail {
    padding: 0 12px 4px 34px;
  }

  .pd-detail-label {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--faint);
    padding: 4px 0 2px;
  }

  .pd-site-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
    width: 100%;
    padding: 2px 0;
    border: none;
    background: none;
    font-family: var(--mono);
    font-size: 11px;
    text-align: left;
    cursor: pointer;
    color: var(--dim);
    transition: background 0.1s;
  }

  .pd-site-row:hover {
    background: var(--surface);
  }

  .pd-site-label {
    color: var(--faint);
    font-size: 10px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .pd-site-op {
    color: var(--green-light);
    white-space: nowrap;
  }

  .pd-site-line {
    margin-left: auto;
    color: var(--faint);
    font-size: 10px;
    flex-shrink: 0;
  }

  /* --- Empty state --- */

  .pd-empty {
    padding: 24px 12px;
    text-align: center;
    color: var(--faint);
    font-family: var(--body);
    font-size: 13px;
  }
</style>
