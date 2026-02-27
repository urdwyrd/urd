<script lang="ts">
  /**
   * DialoguePreview — rendered dialogue preview for a selected section.
   *
   * Reads factSet.choices and factSet.jumps filtered by the selected section.
   * Shows narrator text placeholder, choice buttons (styled), and jump
   * destinations. Selection-driven via selectionContext.
   */

  import { onMount, onDestroy } from 'svelte';
  import { bus } from '$lib/framework/bus/MessageBus';
  import { projectionRegistry } from '$lib/app/projections/ProjectionRegistry';
  import { selectionContext } from '$lib/framework/selection/SelectionContext';
  import type { FactSet, ChoiceFact, JumpEdge } from '$lib/app/compiler/types';

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: null;
    onStateChange: (newState: unknown) => void;
  }

  let { zoneId, zoneTypeId, state: zoneState, onStateChange }: Props = $props();

  let selectedSection: string | null = $state(null);
  let sectionChoices: ChoiceFact[] = $state([]);
  let sectionJumps: JumpEdge[] = $state([]);
  let allSections: string[] = $state([]);

  const unsubscribers: (() => void)[] = [];

  onMount(() => {
    refreshSections();
    unsubscribers.push(bus.subscribe('compiler.completed', refreshSections));
    unsubscribers.push(
      selectionContext.subscribe((state) => {
        const sectionItem = state.items.find((item) => item.kind === 'section');
        if (sectionItem) {
          selectedSection = sectionItem.id;
          refreshChoices();
        }
      }),
    );
  });

  onDestroy(() => {
    for (const unsub of unsubscribers) unsub();
  });

  function refreshSections(): void {
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet) {
      allSections = [];
      sectionChoices = [];
      sectionJumps = [];
      return;
    }

    const sectionIds = new Set<string>();
    for (const choice of factSet.choices) {
      sectionIds.add(choice.section);
    }
    for (const jump of factSet.jumps) {
      sectionIds.add(jump.from_section);
    }
    allSections = [...sectionIds].sort();

    if (selectedSection && !sectionIds.has(selectedSection)) {
      selectedSection = null;
    }
    refreshChoices();
  }

  function refreshChoices(): void {
    const factSet = projectionRegistry.get<FactSet>('urd.projection.factSet');
    if (!factSet || !selectedSection) {
      sectionChoices = [];
      sectionJumps = [];
      return;
    }

    sectionChoices = factSet.choices.filter((c) => c.section === selectedSection);
    sectionJumps = factSet.jumps.filter((j) => j.from_section === selectedSection);
  }

  function handleSectionChange(e: Event): void {
    const target = e.target as HTMLSelectElement;
    selectedSection = target.value || null;
    refreshChoices();
  }

  function jumpTargetLabel(jump: JumpEdge): string {
    if (jump.target.kind === 'end') return 'End dialogue';
    if (jump.target.id) return `Go to: ${jump.target.id}`;
    return jump.target.kind;
  }
</script>

<div class="forge-dialogue-preview">
  <div class="forge-dialogue-preview__toolbar">
    <span class="forge-dialogue-preview__title">Dialogue Preview</span>
    <div class="forge-dialogue-preview__spacer"></div>
    {#if allSections.length > 0}
      <select
        class="forge-dialogue-preview__select"
        value={selectedSection ?? ''}
        onchange={handleSectionChange}
      >
        <option value="">Select section...</option>
        {#each allSections as section}
          <option value={section}>{section}</option>
        {/each}
      </select>
    {/if}
  </div>

  {#if allSections.length === 0}
    <div class="forge-dialogue-preview__empty">
      <p>No dialogue sections available</p>
      <p class="forge-dialogue-preview__hint">Compile a project with dialogue sections to preview them</p>
    </div>
  {:else if !selectedSection}
    <div class="forge-dialogue-preview__empty">
      <p>Select a section to preview</p>
      <p class="forge-dialogue-preview__hint">Use the dropdown above or select a section in another view</p>
    </div>
  {:else}
    <div class="forge-dialogue-preview__content">
      <div class="forge-dialogue-preview__section-name">
        {selectedSection}
      </div>

      <div class="forge-dialogue-preview__narrator">
        <div class="forge-dialogue-preview__narrator-label">Narrator</div>
        <div class="forge-dialogue-preview__narrator-text">
          [Narrative text would appear here during playback]
        </div>
      </div>

      {#if sectionChoices.length > 0}
        <div class="forge-dialogue-preview__choices-header">
          Choices ({sectionChoices.length})
        </div>
        <div class="forge-dialogue-preview__choices">
          {#each sectionChoices as choice}
            <div class="forge-dialogue-preview__choice">
              <span class="forge-dialogue-preview__choice-label">{choice.label}</span>
              <div class="forge-dialogue-preview__choice-meta">
                {#if choice.sticky}
                  <span class="forge-dialogue-preview__tag forge-dialogue-preview__tag--sticky">sticky</span>
                {/if}
                {#if choice.condition_reads.length > 0}
                  <span class="forge-dialogue-preview__tag">
                    {choice.condition_reads.length} condition{choice.condition_reads.length !== 1 ? 's' : ''}
                  </span>
                {/if}
                {#if choice.effect_writes.length > 0}
                  <span class="forge-dialogue-preview__tag forge-dialogue-preview__tag--effect">
                    {choice.effect_writes.length} effect{choice.effect_writes.length !== 1 ? 's' : ''}
                  </span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}

      {#if sectionJumps.length > 0}
        <div class="forge-dialogue-preview__jumps-header">
          Jumps ({sectionJumps.length})
        </div>
        <div class="forge-dialogue-preview__jumps">
          {#each sectionJumps as jump}
            <div class="forge-dialogue-preview__jump">
              → {jumpTargetLabel(jump)}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-dialogue-preview {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
  }

  .forge-dialogue-preview__toolbar {
    display: flex;
    align-items: center;
    gap: var(--forge-space-sm);
    height: 32px;
    padding: 0 var(--forge-space-md);
    background-color: var(--forge-bg-secondary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-dialogue-preview__title {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .forge-dialogue-preview__spacer {
    flex: 1;
  }

  .forge-dialogue-preview__select {
    padding: 1px 4px;
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
    color: var(--forge-text-primary);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-xs);
    max-width: 180px;
  }

  .forge-dialogue-preview__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--forge-space-sm);
    color: var(--forge-text-muted);
  }

  .forge-dialogue-preview__hint {
    font-size: var(--forge-font-size-xs);
    opacity: 0.6;
  }

  .forge-dialogue-preview__content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--forge-space-lg);
  }

  .forge-dialogue-preview__section-name {
    font-size: var(--forge-font-size-lg);
    font-weight: 600;
    color: var(--forge-text-primary);
    margin-bottom: var(--forge-space-lg);
  }

  .forge-dialogue-preview__narrator {
    padding: var(--forge-space-md);
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    margin-bottom: var(--forge-space-lg);
  }

  .forge-dialogue-preview__narrator-label {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--forge-space-xs);
  }

  .forge-dialogue-preview__narrator-text {
    color: var(--forge-text-secondary);
    font-style: italic;
    line-height: 1.5;
    font-family: var(--forge-font-family-body, var(--forge-font-family-ui));
  }

  .forge-dialogue-preview__choices-header,
  .forge-dialogue-preview__jumps-header {
    font-size: var(--forge-font-size-xs);
    font-weight: 600;
    color: var(--forge-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--forge-space-sm);
  }

  .forge-dialogue-preview__choices {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
    margin-bottom: var(--forge-space-lg);
  }

  .forge-dialogue-preview__choice {
    padding: var(--forge-space-sm) var(--forge-space-md);
    border: 1px solid var(--forge-runtime-event-dialogue, #e6a817);
    border-radius: var(--forge-radius-sm);
    background: var(--forge-bg-tertiary);
  }

  .forge-dialogue-preview__choice-label {
    display: block;
    color: var(--forge-text-primary);
    margin-bottom: var(--forge-space-xs);
  }

  .forge-dialogue-preview__choice-meta {
    display: flex;
    gap: var(--forge-space-xs);
    flex-wrap: wrap;
  }

  .forge-dialogue-preview__tag {
    display: inline-block;
    padding: 0 4px;
    border-radius: 2px;
    background: var(--forge-bg-secondary);
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-dialogue-preview__tag--sticky {
    color: var(--forge-runtime-event-set, #4caf50);
  }

  .forge-dialogue-preview__tag--effect {
    color: var(--forge-runtime-event-dialogue, #e6a817);
  }

  .forge-dialogue-preview__jumps {
    display: flex;
    flex-direction: column;
    gap: var(--forge-space-xs);
  }

  .forge-dialogue-preview__jump {
    padding: var(--forge-space-xs) var(--forge-space-sm);
    font-size: var(--forge-font-size-xs);
    color: var(--forge-accent-primary, #5b9bd5);
  }
</style>
