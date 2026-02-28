<script lang="ts">
  /**
   * DialogueOutline — section/choice/jump outline (Section 4).
   * Groups unique section IDs from FactSet choices and jumps.
   */

  import type { FactSet, ChoiceFact, JumpEdge } from '$lib/app/compiler/types';
  import SectionRow from './SectionRow.svelte';

  interface Props {
    factSet: FactSet | null;
    expandedRows: Record<string, boolean>;
    onToggleRow: (key: string) => void;
    onNavigate: (path: string, line: number) => void;
    filterText: string;
  }

  let { factSet, expandedRows, onToggleRow, onNavigate, filterText }: Props = $props();

  interface SectionGroup {
    sectionId: string;
    choices: ChoiceFact[];
    jumps: JumpEdge[];
    earliestLine: number;
  }

  let sections = $derived.by((): SectionGroup[] => {
    if (!factSet) return [];
    const q = filterText.toLowerCase();

    // Collect unique sections
    const sectionMap = new Map<string, { choices: ChoiceFact[]; jumps: JumpEdge[]; earliestLine: number }>();

    for (const choice of factSet.choices) {
      if (q && !choice.section.toLowerCase().includes(q) && !choice.label.toLowerCase().includes(q)) continue;
      let entry = sectionMap.get(choice.section);
      if (!entry) {
        entry = { choices: [], jumps: [], earliestLine: choice.span.start_line };
        sectionMap.set(choice.section, entry);
      }
      entry.choices.push(choice);
      if (choice.span.start_line < entry.earliestLine) {
        entry.earliestLine = choice.span.start_line;
      }
    }

    for (const jump of factSet.jumps) {
      if (q && !jump.from_section.toLowerCase().includes(q)) continue;
      let entry = sectionMap.get(jump.from_section);
      if (!entry) {
        entry = { choices: [], jumps: [], earliestLine: jump.span.start_line };
        sectionMap.set(jump.from_section, entry);
      }
      entry.jumps.push(jump);
      if (jump.span.start_line < entry.earliestLine) {
        entry.earliestLine = jump.span.start_line;
      }
    }

    return [...sectionMap.entries()]
      .map(([sectionId, data]) => ({ sectionId, ...data }))
      .sort((a, b) => a.earliestLine - b.earliestLine);
  });
</script>

<div class="forge-dialogue-outline">
  {#if sections.length === 0}
    <div class="forge-dialogue-outline__empty">No dialogue sections</div>
  {:else if factSet}
    {#each sections as section}
      <SectionRow
        sectionId={section.sectionId}
        choices={section.choices}
        jumps={section.jumps}
        {factSet}
        expanded={expandedRows[`section:${section.sectionId}`] ?? false}
        onToggle={onToggleRow}
        {onNavigate}
      />
    {/each}
  {/if}
</div>

<style>
  .forge-dialogue-outline {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-xs);
  }

  .forge-dialogue-outline__empty {
    padding: var(--forge-space-md);
    color: var(--forge-text-muted);
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    text-align: center;
  }
</style>
