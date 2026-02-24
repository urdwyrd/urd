<script lang="ts">
  import type { FactSet, Diagnostic } from './compiler-bridge';
  import GraphRenderer from './GraphRenderer.svelte';
  import { buildDialogueGraph } from './transform-dialogue';

  interface Props {
    facts: FactSet;
    diagnostics: Diagnostic[];
    onDiagnosticClick?: (line: number, col: number) => void;
  }

  let { facts, diagnostics, onDiagnosticClick }: Props = $props();

  let graph = $derived(buildDialogueGraph(facts, diagnostics));

  function handleNodeClick(nodeId: string) {
    if (!onDiagnosticClick) return;
    // Find the first jump or choice in this section to scroll to.
    const jump = facts.jumps.find((j) => j.from_section === nodeId);
    if (jump) {
      onDiagnosticClick(jump.span.start_line, jump.span.start_col);
      return;
    }
    const choice = facts.choices.find((c) => c.section === nodeId);
    if (choice) {
      onDiagnosticClick(choice.span.start_line, choice.span.start_col);
    }
  }
</script>

{#if facts.jumps.length === 0 && facts.choices.length === 0}
  <div class="graph-empty-state">No dialogue flow in this world.</div>
{:else}
  <GraphRenderer {graph} direction="TB" onNodeClick={handleNodeClick} />
{/if}

<style>
  .graph-empty-state {
    padding: 24px 12px;
    text-align: center;
    color: var(--faint);
    font-family: var(--body);
    font-size: 13px;
  }
</style>
