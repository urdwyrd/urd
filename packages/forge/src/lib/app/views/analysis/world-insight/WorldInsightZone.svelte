<script lang="ts">
  /**
   * WorldInsightZone — unified analysis panel consolidating the compiled
   * world's FactSet, PropertyDependencyIndex, DefinitionIndex, and urdJson
   * into a single interactive surface with 7 collapsible sections.
   */

  import { onMount, onDestroy, untrack } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { navigationBroker } from '$lib/framework/navigation/NavigationBroker';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import { buildInsightState, type InsightState } from './insight-state';
  import { computeFilter, type FilterResult } from './insight-filter';
  import InsightSection from './InsightSection.svelte';
  import WorldSummary from './WorldSummary.svelte';
  import EntityAtlas from './EntityAtlas.svelte';
  import PropertyFlow from './PropertyFlow.svelte';
  import DialogueOutline from './DialogueOutline.svelte';
  import LocationMap from './LocationMap.svelte';
  import RulesEngine from './RulesEngine.svelte';
  import DiagnosticList from './DiagnosticList.svelte';

  type FilterMode = 'all' | 'read_only' | 'write_only' | 'orphaned';

  interface ZoneState {
    collapsedSections: Record<string, boolean>;
    filterText: string;
    expandedRows: Record<string, boolean>;
    expandedDetails: Record<string, boolean>;
    propertyFilterMode: FilterMode;
    entityTypeFilter: string | null;
  }

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: ZoneState;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let data: InsightState = $state({
    world: null,
    factSet: null,
    propertyIndex: null,
    diagnosticsByFile: null,
    entityTable: null,
    symbolTable: null,
    worldStats: null,
    entityLocationMap: new Map(),
    symbolMap: new Map(),
  });

  let highlightedEntityId: string | null = $state(null);
  let filterInput: string = $state(untrack(() => zoneState.filterText ?? ''));
  let filterDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let rootEl: HTMLDivElement | undefined = $state(undefined);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshData();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshData));
    unsubscribers.push(selectionContext.subscribe(handleSelectionChange));
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
    if (filterDebounceTimer) clearTimeout(filterDebounceTimer);
  });

  function refreshData(): void {
    data = buildInsightState();
  }

  // ---- State management ----

  function updateState(patch: Partial<ZoneState>): void {
    onStateChange({ ...zoneState, ...patch });
  }

  function toggleSection(sectionId: string): void {
    const collapsed = { ...zoneState.collapsedSections };
    collapsed[sectionId] = !collapsed[sectionId];
    updateState({ collapsedSections: collapsed });
  }

  function toggleRow(key: string): void {
    const expanded = { ...zoneState.expandedRows };
    expanded[key] = !expanded[key];
    updateState({ expandedRows: expanded });
  }

  function isCollapsed(sectionId: string): boolean {
    return zoneState.collapsedSections[sectionId] ?? false;
  }

  function setPropertyFilterMode(mode: FilterMode): void {
    updateState({ propertyFilterMode: mode });
  }

  function setEntityTypeFilter(type: string | null): void {
    updateState({ entityTypeFilter: type });
  }

  // ---- Navigation ----

  function navigateToSource(path: string, line: number): void {
    navigationBroker.navigate({
      targetViewId: 'urd.codeEditor',
      params: { path, line },
    });
  }

  function scrollToSection(sectionId: string): void {
    // Ensure the section is expanded
    if (zoneState.collapsedSections[sectionId]) {
      toggleSection(sectionId);
    }
    // Scroll into view
    const el = rootEl?.querySelector(`[data-section-id="${sectionId}"]`);
    if (el) {
      el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }

  // ---- Global filter ----

  function handleFilterInput(): void {
    if (filterDebounceTimer) clearTimeout(filterDebounceTimer);
    filterDebounceTimer = setTimeout(() => {
      updateState({ filterText: filterInput });
    }, 150);
  }

  function clearFilter(): void {
    filterInput = '';
    highlightedEntityId = null;
    updateState({ filterText: '', entityTypeFilter: null });
  }

  let filterResult: FilterResult | null = $derived.by(() => {
    if (!zoneState.filterText) return null;
    return computeFilter(zoneState.filterText, data);
  });

  // ---- Selection context ----

  function handleSelectionChange(selState: { items: { kind: string; id: string }[]; sourceZoneId: string | null }): void {
    if (selState.sourceZoneId === zoneId) return;
    if (selState.items.length > 0 && selState.items[0].kind === 'entity') {
      const entityId = selState.items[0].id;
      highlightedEntityId = entityId;

      // Auto-expand the entity in the atlas
      const entity = data.world?.entities.find((e) => e.id === entityId);
      if (entity?.type) {
        const expanded = { ...zoneState.expandedRows };
        expanded[`type:${entity.type}`] = true;
        expanded[`entity:${entityId}`] = true;
        updateState({ expandedRows: expanded });

        // Also filter properties to this entity's type
        updateState({ entityTypeFilter: entity.type });
      }
    } else {
      highlightedEntityId = null;
    }
  }

  // ---- Keyboard navigation ----

  function handleKeydown(e: KeyboardEvent): void {
    // Only handle if the panel has focus
    if (!rootEl?.contains(document.activeElement)) return;

    const target = document.activeElement;
    if (!target || target === rootEl) return;

    const rows = rootEl.querySelectorAll('[class*="__header"], [class*="__row"]');
    const rowArray = Array.from(rows) as HTMLElement[];
    const currentIdx = rowArray.indexOf(target as HTMLElement);
    if (currentIdx === -1) return;

    switch (e.key) {
      case 'ArrowDown': {
        e.preventDefault();
        const next = rowArray[currentIdx + 1];
        if (next) next.focus();
        break;
      }
      case 'ArrowUp': {
        e.preventDefault();
        const prev = rowArray[currentIdx - 1];
        if (prev) prev.focus();
        break;
      }
    }
  }

  // ---- Badge counts ----

  let entityCount = $derived(data.world?.entities.length ?? 0);
  let locationCount = $derived(data.world?.locations.length ?? 0);
  let sectionCount = $derived.by(() => {
    if (!data.factSet) return 0;
    const sections = new Set<string>();
    for (const c of data.factSet.choices) sections.add(c.section);
    for (const j of data.factSet.jumps) sections.add(j.from_section);
    return sections.size;
  });
  let choiceCount = $derived(data.factSet?.choices.length ?? 0);
  let exitCount = $derived(data.factSet?.exits.length ?? 0);
  let ruleCount = $derived(data.world?.rules ? Object.keys(data.world.rules).length : 0);
  let diagnosticCount = $derived(data.worldStats?.diagnosticCount ?? 0);
  let propertyCount = $derived(data.propertyIndex?.summary.total_properties ?? 0);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="forge-world-insight"
  bind:this={rootEl}
  onkeydown={handleKeydown}
>
  {#if data.world}
    <!-- Global filter -->
    <div class="forge-world-insight__filter-bar">
      <input
        type="text"
        class="forge-world-insight__filter-input"
        placeholder="Filter... (e.g. @innkeeper)"
        bind:value={filterInput}
        oninput={handleFilterInput}
      />
      {#if zoneState.filterText || highlightedEntityId}
        <button class="forge-world-insight__filter-clear" onclick={clearFilter}>
          Clear
        </button>
      {/if}
    </div>

    <!-- Section 1: World Summary (always visible) -->
    <WorldSummary
      world={data.world}
      factSet={data.factSet}
      worldStats={data.worldStats}
      onBadgeClick={scrollToSection}
      onNavigate={navigateToSource}
    />

    <!-- Section 2: Entity Atlas -->
    <InsightSection
      sectionId="entities"
      title="ENTITIES"
      collapsed={isCollapsed('entities')}
      onToggle={toggleSection}
      badges={[{ label: '', count: entityCount }]}
    >
      <EntityAtlas
        world={data.world}
        symbolMap={data.symbolMap}
        propertyIndex={data.propertyIndex}
        factSet={data.factSet}
        entityLocationMap={data.entityLocationMap}
        expandedRows={zoneState.expandedRows}
        onToggleRow={toggleRow}
        onNavigate={navigateToSource}
        filterText={zoneState.filterText}
        entityTypeFilter={zoneState.entityTypeFilter}
        onEntityTypeFilterChange={setEntityTypeFilter}
        {highlightedEntityId}
      />
    </InsightSection>

    <!-- Section 3: Property Flow -->
    <InsightSection
      sectionId="properties"
      title="PROPERTIES"
      collapsed={isCollapsed('properties')}
      onToggle={toggleSection}
      badges={[{ label: '', count: propertyCount }]}
    >
      <PropertyFlow
        propertyIndex={data.propertyIndex}
        factSet={data.factSet}
        expandedRows={zoneState.expandedRows}
        onToggleRow={toggleRow}
        onNavigate={navigateToSource}
        filterText={zoneState.filterText}
        filterMode={zoneState.propertyFilterMode ?? 'all'}
        onFilterModeChange={setPropertyFilterMode}
      />
    </InsightSection>

    <!-- Section 4: Dialogue Outline -->
    <InsightSection
      sectionId="dialogue"
      title="DIALOGUE"
      collapsed={isCollapsed('dialogue')}
      onToggle={toggleSection}
      badges={[
        { label: '', count: sectionCount },
        { label: 'choices', count: choiceCount },
      ]}
    >
      <DialogueOutline
        factSet={data.factSet}
        expandedRows={zoneState.expandedRows}
        onToggleRow={toggleRow}
        onNavigate={navigateToSource}
        filterText={zoneState.filterText}
      />
    </InsightSection>

    <!-- Section 5: Location Map -->
    <InsightSection
      sectionId="locations"
      title="LOCATIONS"
      collapsed={isCollapsed('locations')}
      onToggle={toggleSection}
      badges={[
        { label: '', count: locationCount },
        { label: 'exits', count: exitCount },
      ]}
    >
      <LocationMap
        world={data.world}
        symbolMap={data.symbolMap}
        entityLocationMap={data.entityLocationMap}
        expandedRows={zoneState.expandedRows}
        onToggleRow={toggleRow}
        onNavigate={navigateToSource}
        filterText={zoneState.filterText}
      />
    </InsightSection>

    <!-- Section 6: Rules Engine -->
    {#if ruleCount > 0}
      <InsightSection
        sectionId="rules"
        title="RULES"
        collapsed={isCollapsed('rules')}
        onToggle={toggleSection}
        badges={[{ label: '', count: ruleCount }]}
      >
        <RulesEngine
          world={data.world}
          factSet={data.factSet}
          expandedRows={zoneState.expandedRows}
          onToggleRow={toggleRow}
          onNavigate={navigateToSource}
          filterText={zoneState.filterText}
        />
      </InsightSection>
    {/if}

    <!-- Section 7: Diagnostics -->
    <InsightSection
      sectionId="diagnostics"
      title="DIAGNOSTICS"
      collapsed={isCollapsed('diagnostics')}
      onToggle={toggleSection}
      badges={[
        { label: '', count: diagnosticCount, colour: diagnosticCount > 0 ? 'var(--forge-status-warning)' : undefined },
      ]}
    >
      <DiagnosticList
        diagnosticsByFile={data.diagnosticsByFile}
        filterText={zoneState.filterText}
        onNavigate={navigateToSource}
      />
    </InsightSection>
  {:else}
    <div class="forge-world-insight__empty">
      No compilation data available
    </div>
  {/if}
</div>

<style>
  .forge-world-insight {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    font-family: var(--forge-font-family-ui);
  }

  .forge-world-insight__filter-bar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    padding: var(--forge-space-xs) var(--forge-space-sm);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-world-insight__filter-input {
    flex: 1;
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
    background-color: var(--forge-bg-secondary);
    color: var(--forge-text-primary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    padding: 3px var(--forge-space-xs);
    outline: none;
  }

  .forge-world-insight__filter-input::placeholder {
    color: var(--forge-text-muted);
  }

  .forge-world-insight__filter-input:focus {
    border-color: var(--forge-text-muted);
  }

  .forge-world-insight__filter-clear {
    font-family: var(--forge-font-family-ui);
    font-size: 10px;
    padding: 2px var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    background: none;
    border: 1px solid var(--forge-border-zone);
    color: var(--forge-text-muted);
    cursor: pointer;
  }

  .forge-world-insight__filter-clear:hover {
    color: var(--forge-text-primary);
    border-color: var(--forge-text-muted);
  }

  .forge-world-insight__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: var(--forge-text-muted);
    font-size: var(--forge-font-size-md);
  }
</style>
